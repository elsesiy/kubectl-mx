use assert_cmd::Command;
use std::env;
use std::fs;

#[test]
fn test_dry_run_no_match() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.arg("-r")
        .arg("nonexistent")
        .arg("-d")
        .arg("-e")
        .arg("get")
        .arg("pods");
    cmd.assert()
        .success()
        .stdout("No contexts match the provided regex.\n");
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn test_help_shows_defaults() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("[default: 10]"))
        .stdout(predicates::str::contains("[default: 30]"))
        .stdout(predicates::str::contains("[default: 0]"));
}

#[test]
fn test_env_override_in_help() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.env("KUBECTL_MX_MAX_CONCURRENCY", "20");
    cmd.arg("--help");
    cmd.assert().success().stdout(predicates::str::contains(
        "[env: KUBECTL_MX_MAX_CONCURRENCY=20]",
    ));
}

#[test]
fn test_config_file_is_loaded() {
    let temp_dir = env::temp_dir().join("kubectl_mx_test_config");
    fs::create_dir_all(&temp_dir).unwrap();
    let config_dir = temp_dir.join("kubectl-mx");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config.toml");
    fs::write(&config_file, "max_concurrency = 25\ntimeout = 60\n").unwrap();

    // Verify the binary starts successfully with a config file present
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.to_str().unwrap());
    cmd.env_remove("KUBECTL_MX_MAX_CONCURRENCY");
    cmd.env_remove("KUBECTL_MX_TIMEOUT");
    cmd.env_remove("KUBECTL_MX_RETRY");
    cmd.arg("-r")
        .arg("nonexistent")
        .arg("-d")
        .arg("-e")
        .arg("get")
        .arg("pods");
    cmd.assert()
        .success()
        .stdout("No contexts match the provided regex.\n");

    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_dry_run_with_matches() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.arg("-r")
        .arg("kind-mx-test.*")
        .arg("-d")
        .arg("-e")
        .arg("get")
        .arg("pods");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Found 3 matching context(s)"));
}

#[test]
fn test_exit_code_nonzero_when_kubectl_fails() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.args([
        "-r",
        "kind-mx-test.*",
        "-q",
        "-e",
        "get",
        "nonexistent-resource",
    ]);
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains("0 succeeded"));
}

#[test]
fn test_exit_code_zero_when_kubectl_succeeds() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.args(["-r", "kind-mx-test.*", "-q", "-e", "get", "namespaces"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("0 failed"));
}

#[test]
fn test_zero_max_concurrency_is_rejected() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.args(["-r", ".*", "--max-concurrency", "0", "-e", "get", "pods"]);
    cmd.assert()
        .code(2)
        .stderr(predicates::str::contains("must be at least 1"));
}

#[test]
fn test_no_match_without_dry_run_fails() {
    let mut cmd = Command::cargo_bin("kubectl-mx").unwrap();
    cmd.args(["-r", "nonexistent", "-e", "get", "pods"]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("No contexts match"));
}
