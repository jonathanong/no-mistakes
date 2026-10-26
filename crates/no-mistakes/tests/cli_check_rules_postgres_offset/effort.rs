use super::*;

fn report(config: &str, effort: Option<&str>) -> serde_json::Value {
    let root = fixture("effort");
    let mut command = Command::new(bin());
    command
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config", config, "--format", "json"]);
    if let Some(effort) = effort {
        command.args(["--effort", effort]);
    }
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", stdout(&output));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn saved_offset_efforts_preserve_default_and_cli_overrides() {
    let max = report(".no-mistakes.max.yml", None);
    assert_eq!(max, report(".no-mistakes.yml", None));
    let low = report(".no-mistakes.low.yml", None);
    assert!(max["rules"].as_array().unwrap().len() >= low["rules"].as_array().unwrap().len());
    assert_eq!(low, report(".no-mistakes.max.yml", Some("low")));
    assert_eq!(max, report(".no-mistakes.low.yml", Some("max")));
    let advise = report(".no-mistakes.low-advise.yml", None);
    assert_eq!(
        advise["advisories"].as_array().unwrap().len(),
        3,
        "{advise}"
    );
    assert_eq!(
        advise["rules"],
        report(".no-mistakes.low-ignore.yml", None)["rules"]
    );
}
