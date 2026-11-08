use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub fn check_report(root: &Path) -> Result<Value, String> {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--jobs", "1", "check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json", "--effort", "max"])
        .output()
        .map_err(|error| error.to_string())?;
    serde_json::from_slice(&output.stdout).map_err(|error| {
        format!(
            "check JSON: {error}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

pub fn run_rule(root: &Path, rule_id: &str) -> Result<Vec<String>, String> {
    let report = check_report(root)?;
    let mut hits: Vec<_> = ["rules", "codebase", "advisories"]
        .into_iter()
        .filter_map(|channel| report[channel].as_array())
        .flatten()
        .filter(|finding| finding["rule"].as_str() == Some(rule_id))
        .map(|finding| {
            format!(
                "{}:{}:{}",
                finding["file"].as_str().unwrap_or_default(),
                finding["line"].as_u64().unwrap_or_default(),
                finding["message"].as_str().unwrap_or_default(),
            )
        })
        .collect();
    if hits.is_empty()
        && report["incomplete"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    {
        return Err(format!(
            "incomplete snippet analysis: {}",
            report["incomplete"]
        ));
    }
    hits.sort();
    hits.dedup();
    Ok(hits)
}
