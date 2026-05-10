use clap::{Arg, ArgMatches, Command as ClapCommand};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Default)]
pub struct Config {
    pub max_concurrency: Option<usize>,
    pub timeout: Option<u64>,
    pub retry: Option<usize>,
}

impl Config {
    pub fn load() -> Self {
        let config_dir = env::var("XDG_CONFIG_HOME")
            .unwrap_or_else(|_| env::var("HOME").unwrap_or_else(|_| ".".to_string()) + "/.config");
        let config_path = Path::new(&config_dir)
            .join("kubectl-mx")
            .join("config.toml");

        fs::read_to_string(config_path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Returns the effective max_concurrency value, considering CLI > env > config > default.
    pub fn max_concurrency(&self, matches: &ArgMatches) -> usize {
        if matches.value_source("max-concurrency") != Some(clap::parser::ValueSource::DefaultValue)
        {
            return *matches.get_one::<usize>("max-concurrency").unwrap();
        }
        self.max_concurrency.unwrap_or(10)
    }

    /// Returns the effective timeout value, considering CLI > env > config > default.
    pub fn timeout(&self, matches: &ArgMatches) -> u64 {
        if matches.value_source("timeout") != Some(clap::parser::ValueSource::DefaultValue) {
            return *matches.get_one::<u64>("timeout").unwrap();
        }
        self.timeout.unwrap_or(30)
    }

    /// Returns the effective retry value, considering CLI > env > config > default.
    pub fn retry(&self, matches: &ArgMatches) -> usize {
        if matches.value_source("retry") != Some(clap::parser::ValueSource::DefaultValue) {
            return *matches.get_one::<usize>("retry").unwrap();
        }
        self.retry.unwrap_or(0)
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
                .value_parser(clap::value_parser!(usize))
                .default_value("10"),
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .help("Timeout for each kubectl command in seconds")
                .value_name("SECONDS")
                .env("KUBECTL_MX_TIMEOUT")
                .value_parser(clap::value_parser!(u64))
                .default_value("30"),
        )
        .arg(
            Arg::new("retry")
                .long("retry")
                .help("Number of retries for failed commands")
                .value_name("N")
                .env("KUBECTL_MX_RETRY")
                .value_parser(clap::value_parser!(usize))
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
