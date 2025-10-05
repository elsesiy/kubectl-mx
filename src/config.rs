use clap::{Arg, Command as ClapCommand};
use std::env;
use std::fs;
use std::path::Path;

pub fn load_config() {
    let config_dir = env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| env::var("HOME").unwrap_or_else(|_| ".".to_string()) + "/.config");
    let config_path = Path::new(&config_dir)
        .join("kubectl-mx")
        .join("config.toml");
    if let Ok(cfg_str) = fs::read_to_string(config_path) {
        if let Ok(cfg) = toml::from_str::<toml::Value>(&cfg_str) {
            if let Some(v) = cfg.get("max_concurrency") {
                if env::var("KUBECTL_MX_MAX_CONCURRENCY").is_err() {
                    env::set_var("KUBECTL_MX_MAX_CONCURRENCY", v.to_string());
                }
            }
            if let Some(v) = cfg.get("timeout") {
                if env::var("KUBECTL_MX_TIMEOUT").is_err() {
                    env::set_var("KUBECTL_MX_TIMEOUT", v.to_string());
                }
            }
            if let Some(v) = cfg.get("retry") {
                if env::var("KUBECTL_MX_RETRY").is_err() {
                    env::set_var("KUBECTL_MX_RETRY", v.to_string());
                }
            }
        }
    }
}

pub fn build_command() -> ClapCommand {
    ClapCommand::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .trailing_var_arg(true)
        .arg(
            Arg::new("regex")
                .short('r')
                .long("regex")
                .help("Regex to select which kubectl contexts to use")
                .value_name("REGEX")
                .required(true),
        )
        .arg(
            Arg::new("dry_run")
                .short('d')
                .long("dry-run")
                .help("Outputs matching contexts based on regex provided")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("exec")
                .short('e')
                .long("exec")
                .help("kubectl command to execute against matching clusters")
                .allow_hyphen_values(true)
                .value_name("CMD")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("max-concurrency")
                .long("max-concurrency")
                .help("Maximum number of concurrent kubectl commands")
                .value_name("N")
                .env("KUBECTL_MX_MAX_CONCURRENCY")
                .default_value("10"),
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .help("Timeout for each kubectl command in seconds")
                .value_name("SECONDS")
                .env("KUBECTL_MX_TIMEOUT")
                .default_value("30"),
        )
        .arg(
            Arg::new("retry")
                .long("retry")
                .help("Number of retries for failed commands")
                .value_name("N")
                .env("KUBECTL_MX_RETRY")
                .default_value("0"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress kubectl output, only show progress and summary")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output results in JSON format")
                .action(clap::ArgAction::SetTrue),
        )
}
