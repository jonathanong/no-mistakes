use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub fn check_report(root: &Path) -> Result<Value, String> {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .env(
            "CARGO_BIN_EXE_no-mistakes",
            env!("CARGO_BIN_EXE_no-mistakes"),
        )
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

pub fn execute_rule_snippets(root: &Path, rule_id: &str) -> Vec<String> {
    let page = super::load_page(root, rule_id);
    let mut valid = super::heading_section(&page.body, "Valid example");
    if super::fences(&valid).is_empty() {
        valid = super::labeled_section(&page.body, &["Valid example", "Pass"]);
    }
    let mut counter = super::counterexample_section(&page.body);
    if super::fences(&counter).is_empty() {
        counter = super::labeled_section(&page.body, &["Counterexample", "Fail"]);
    }
    let valid_fences = super::fences(&valid);
    let counter_fences = super::fences(&counter);
    if valid_fences.is_empty() {
        return vec![format!(
            "{}: Valid example needs an executable fence",
            page.file
        )];
    }
    if counter_fences.is_empty() {
        return vec![format!(
            "{}: Counterexample needs an executable fence",
            page.file
        )];
    }
    let config = super::rule_config_yaml(&page.body, rule_id);
    let mut failures = Vec::new();
    for (label, expect_report, snippet) in [
        ("Valid example", false, valid_fences.as_slice()),
        ("Counterexample", true, counter_fences.as_slice()),
    ] {
        for (index, fences) in super::snippet_runs(snippet) {
            let fixture = super::materialize_snippet(&config, &fences);
            match run_rule(fixture.path(), rule_id) {
                Ok(hits) if hits.is_empty() == expect_report => failures.push(format!(
                    "{}: {label}[{index}] expected {} but hits={hits:?}",
                    page.file,
                    if expect_report {
                        "a finding"
                    } else {
                        "silence"
                    },
                )),
                Ok(_) => {}
                Err(err) => {
                    failures.push(format!("{}: {label}[{index}] run error: {err}", page.file))
                }
            }
        }
    }
    failures
}
