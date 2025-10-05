//! Core functionality for kubectl-mx

use regex::Regex;

/// Formats the dry-run output showing matching contexts and command template.
///
/// # Arguments
/// * `ctxs` - Slice of matching context names
/// * `cmd` - Slice of command arguments
///
/// # Returns
/// A formatted string for dry-run output
pub fn format_dry_run(ctxs: &[String], cmd: &[String]) -> String {
    if ctxs.is_empty() {
        "No contexts match the provided regex.".to_string()
    } else {
        let mut lines = vec![format!("Found {} matching context(s):", ctxs.len())];
        let max_display = 10;
        for (i, ctx) in ctxs.iter().enumerate() {
            if i >= max_display {
                let remaining = ctxs.len() - max_display;
                lines.push(format!("... and {} more", remaining));
                break;
            }
            lines.push(format!("- {}", ctx));
        }
        lines.push("".to_string());
        lines.push(format!(
            "Command to execute: kubectl --context <context> {}",
            cmd.join(" ")
        ));
        lines.join("\n")
    }
}

/// Retrieves all available kubectl contexts.
///
/// # Returns
/// A Result containing a string with newline-separated context names, or an error
pub fn get_kubectl_contexts() -> Result<String, Box<dyn std::error::Error>> {
    let proc_output = std::process::Command::new("kubectl")
        .arg("config")
        .arg("get-contexts")
        .arg("-o")
        .arg("name")
        .output()?;

    if !proc_output.status.success() {
        return Err(format!(
            "kubectl command failed: {}",
            String::from_utf8_lossy(&proc_output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8(proc_output.stdout)?)
}

/// Filters kubectl contexts using a regex pattern.
///
/// # Arguments
/// * `regex` - The regex pattern to match against context names
/// * `output` - The output from kubectl config get-contexts -o name
///
/// # Returns
/// A Result containing a vector of matching context names, or an error
pub fn get_matching_contexts(
    regex: String,
    output: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let re = Regex::new(&regex).unwrap();

    let filtered_contexts: Vec<String> = output
        .lines()
        .filter(|l| re.is_match(l))
        .map(|m| m.to_string())
        .collect();

    Ok(filtered_contexts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_matching_contexts() {
        let mock_output = "context1\ncontext2-prod\ncontext3-dev\ncontext4-prod";

        // Test regex that matches prod contexts
        let result = get_matching_contexts("prod".to_string(), mock_output).unwrap();
        assert_eq!(
            result,
            vec!["context2-prod".to_string(), "context4-prod".to_string()]
        );

        // Test regex that matches all
        let result = get_matching_contexts("context".to_string(), mock_output).unwrap();
        assert_eq!(
            result,
            vec![
                "context1".to_string(),
                "context2-prod".to_string(),
                "context3-dev".to_string(),
                "context4-prod".to_string()
            ]
        );

        // Test regex that matches none
        let result = get_matching_contexts("nonexistent".to_string(), mock_output).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_format_dry_run_no_contexts() {
        let ctxs: Vec<String> = vec![];
        let cmd = vec!["get".to_string(), "pods".to_string()];
        let output = format_dry_run(&ctxs, &cmd);
        assert_eq!(output, "No contexts match the provided regex.");
    }

    #[test]
    fn test_format_dry_run_few_contexts() {
        let ctxs = vec!["ctx1".to_string(), "ctx2".to_string()];
        let cmd = vec!["get".to_string(), "pods".to_string()];
        let output = format_dry_run(&ctxs, &cmd);
        let expected = "Found 2 matching context(s):\n- ctx1\n- ctx2\n\nCommand to execute: kubectl --context <context> get pods";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_format_dry_run_many_contexts() {
        let ctxs: Vec<String> = (1..=15).map(|i| format!("context{}", i)).collect();
        let cmd = vec![
            "describe".to_string(),
            "deployment".to_string(),
            "app".to_string(),
        ];
        let output = format_dry_run(&ctxs, &cmd);
        let expected = "Found 15 matching context(s):\n- context1\n- context2\n- context3\n- context4\n- context5\n- context6\n- context7\n- context8\n- context9\n- context10\n... and 5 more\n\nCommand to execute: kubectl --context <context> describe deployment app";
        assert_eq!(output, expected);
    }
}
