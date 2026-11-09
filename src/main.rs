//! kubectl-mx: Execute kubectl commands across multiple contexts
//!
//! This tool allows running kubectl commands on multiple Kubernetes contexts
//! that match a given regex pattern, with parallel execution for efficiency.

use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use kubectl_mx::{format_dry_run, get_kubectl_contexts, get_matching_contexts};
use serde::Serialize;
use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{Duration, timeout};

mod config;

#[derive(Serialize)]
struct DryRunResult {
    matching_contexts: Vec<String>,
    command: String,
}

#[derive(Serialize, Clone)]
struct ExecutionResult {
    context: String,
    success: bool,
    output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
    error: Option<String>,
}

/// Writes a line to stdout or stderr without corrupting the progress bar.
///
/// Write errors (such as a closed pipe from `| head`) are ignored on purpose: aborting would
/// leave a mutating command applied to only some of the contexts.
fn emit(pb: &Option<ProgressBar>, to_stderr: bool, msg: &str) {
    let write = || {
        if to_stderr {
            let _ = writeln!(std::io::stderr(), "{msg}");
        } else {
            let _ = writeln!(std::io::stdout(), "{msg}");
        }
    };
    match pb {
        Some(pb) => pb.suspend(write),
        None => write(),
    }
}

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let cfg = config::Config::load();
    let matches = config::build_command().get_matches();

    let var_args: Arc<Vec<String>> = Arc::new(
        matches
            .get_many::<String>("exec")
            .map(|m| m.cloned().collect())
            .unwrap_or_default(),
    );

    let json = matches.get_flag("json");

    let max_concurrency = cfg.max_concurrency(&matches);
    let timeout_secs = cfg.timeout(&matches);
    let retry_count = cfg.retry(&matches);
    // A zero permit semaphore never grants a permit and a zero timeout fails every command.
    if max_concurrency == 0 || timeout_secs == 0 {
        emit(
            &None,
            true,
            "max-concurrency and timeout must be at least 1",
        );
        return Ok(ExitCode::from(2));
    }

    let r_arg = matches.get_one::<String>("regex").unwrap();
    let output = get_kubectl_contexts()?;
    let ctxs = get_matching_contexts(r_arg, &output)?;

    if matches.get_flag("dry_run") || ctxs.is_empty() {
        // Nothing ran although the user asked for execution, so scripts must see a failure.
        let no_match = ctxs.is_empty() && !matches.get_flag("dry_run");
        if json {
            let result = DryRunResult {
                matching_contexts: ctxs,
                command: format!("kubectl --context <context> {}", var_args.join(" ")),
            };
            emit(&None, false, &serde_json::to_string_pretty(&result)?);
        } else {
            emit(&None, no_match, &format_dry_run(&ctxs, &var_args));
        }
        return Ok(if no_match {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        });
    }

    // Without a terminal nobody can answer the prompt, so run like --json does.
    let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    if !json
        && interactive
        && ctxs.len() > 20
        && !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Found {} matching contexts. Proceed?", ctxs.len()))
            .default(true)
            .interact()?
    {
        return Ok(ExitCode::SUCCESS);
    }

    let quiet = matches.get_flag("quiet");
    let start_time = std::time::Instant::now();
    let semaphore = Arc::new(Semaphore::new(max_concurrency));

    let pb = if json {
        None
    } else {
        let pb = ProgressBar::new(ctxs.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
        Some(pb)
    };

    let ctxs_len = ctxs.len();
    let mut tasks = Vec::with_capacity(ctxs_len);
    for c in ctxs {
        let var_args = var_args.clone();
        let semaphore = semaphore.clone();
        let pb = pb.clone();
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let mut exec_result = ExecutionResult {
                context: c,
                success: false,
                output: None,
                stderr: None,
                error: None,
            };
            for attempt in 0..=retry_count {
                let out = timeout(
                    Duration::from_secs(timeout_secs),
                    tokio::process::Command::new("kubectl")
                        .arg("--context")
                        .arg(&exec_result.context)
                        .args(&*var_args)
                        // Dropping the future on timeout must not leave kubectl running.
                        .kill_on_drop(true)
                        .output(),
                )
                .await;

                let attempt_result = match out {
                    Ok(Ok(output)) => {
                        if output.status.success() {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            exec_result.success = true;
                            if json {
                                exec_result.output = Some(stdout);
                                exec_result.stderr = (!stderr.is_empty()).then_some(stderr);
                            } else if !quiet {
                                // One print call so output of concurrent contexts cannot interleave.
                                emit(
                                    &pb,
                                    false,
                                    &format!(
                                        "{}\n----------\n{}",
                                        exec_result.context.green(),
                                        stdout
                                    ),
                                );
                                if !stderr.is_empty() {
                                    emit(
                                        &pb,
                                        true,
                                        &format!(
                                            "{}: {}",
                                            exec_result.context.yellow(),
                                            stderr.trim_end()
                                        ),
                                    );
                                }
                            }
                            Ok(())
                        } else {
                            Err(String::from_utf8_lossy(&output.stderr).to_string())
                        }
                    }
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(_) => Err("Command timed out".to_string()),
                };

                match attempt_result {
                    Ok(()) => break,
                    Err(err) => {
                        if attempt < retry_count {
                            if !json {
                                emit(
                                    &pb,
                                    true,
                                    &format!(
                                        "Error in context {} (attempt {}): {}",
                                        exec_result.context.red(),
                                        attempt + 1,
                                        err
                                    ),
                                );
                            }
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        } else {
                            if !json {
                                emit(
                                    &pb,
                                    true,
                                    &format!(
                                        "Error in context {}: {}",
                                        exec_result.context.red(),
                                        err
                                    ),
                                );
                            }
                            exec_result.error = Some(err);
                            break;
                        }
                    }
                }
            }

            if let Some(pb) = &pb {
                pb.inc(1);
            }
            exec_result
        });
        tasks.push(task);
    }

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut all_results = Vec::with_capacity(ctxs_len);
    for task in tasks {
        let result = task.await.unwrap();
        if result.success {
            success_count += 1;
        } else {
            failure_count += 1;
        }
        all_results.push(result);
    }

    if let Some(pb) = pb {
        pb.finish();
    }
    let elapsed = start_time.elapsed();
    if json {
        let output = serde_json::json!({
            "results": all_results,
            "summary": {
                "total": ctxs_len,
                "succeeded": success_count,
                "failed": failure_count,
                "elapsed_seconds": elapsed.as_secs_f64()
            }
        });
        emit(&None, false, &serde_json::to_string_pretty(&output)?);
    } else {
        emit(
            &None,
            false,
            &format!(
                "Completed: {} succeeded, {} failed in {:.2}s",
                success_count.to_string().green(),
                failure_count.to_string().red(),
                elapsed.as_secs_f64()
            ),
        );
    }

    Ok(if failure_count > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
