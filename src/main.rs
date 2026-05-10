//! kubectl-mx: Execute kubectl commands across multiple contexts
//!
//! This tool allows running kubectl commands on multiple Kubernetes contexts
//! that match a given regex pattern, with parallel execution for efficiency.

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use kubectl_mx::{format_dry_run, get_kubectl_contexts, get_matching_contexts};
use serde::Serialize;
use std::process;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

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
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::Config::load();
    let matches = config::build_command().get_matches();

    let mut vargs: Vec<String> = matches
        .get_many::<String>("exec")
        .map(|m| m.cloned().collect())
        .unwrap_or_default();

    let json = matches.get_flag("json");

    if json && !vargs.iter().any(|a| a == "-o" || a == "--output") {
        vargs.push("-o".to_string());
        vargs.push("json".to_string());
    }

    let var_args = Arc::new(vargs);

    let r_arg = matches.get_one::<String>("regex").unwrap();
    let output = get_kubectl_contexts()?;
    let ctxs = get_matching_contexts(r_arg, &output)?;

    if matches.get_flag("dry_run") || ctxs.is_empty() {
        if json {
            let result = DryRunResult {
                matching_contexts: ctxs,
                command: format!("kubectl --context <context> {}", var_args.join(" ")),
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{}", format_dry_run(&ctxs, &var_args));
        }
        process::exit(0);
    }

    if !json
        && ctxs.len() > 20
        && !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Found {} matching contexts. Proceed?", ctxs.len()))
            .default(true)
            .interact()
            .unwrap()
    {
        process::exit(0);
    }

    let max_concurrency = cfg.max_concurrency(&matches);
    let timeout_secs = cfg.timeout(&matches);
    let retry_count = cfg.retry(&matches);
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
                error: None,
            };
            for attempt in 0..=retry_count {
                let out = timeout(
                    Duration::from_secs(timeout_secs),
                    tokio::process::Command::new("kubectl")
                        .arg("--context")
                        .arg(&exec_result.context)
                        .args(&*var_args)
                        .output(),
                )
                .await;

                let attempt_result = match out {
                    Ok(Ok(output)) => {
                        if output.status.success() {
                            if !quiet && !json {
                                println!("{}\n----------", exec_result.context.green());
                                println!("{}", String::from_utf8_lossy(&output.stdout));
                            }
                            exec_result.success = true;
                            if json {
                                exec_result.output =
                                    Some(String::from_utf8_lossy(&output.stdout).to_string());
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
                                eprintln!(
                                    "Error in context {} (attempt {}): {}",
                                    exec_result.context.red(),
                                    attempt + 1,
                                    err
                                );
                            }
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        } else {
                            if !json {
                                eprintln!(
                                    "Error in context {}: {}",
                                    exec_result.context.red(),
                                    err
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
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "Completed: {} succeeded, {} failed in {:.2}s",
            success_count.to_string().green(),
            failure_count.to_string().red(),
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}
