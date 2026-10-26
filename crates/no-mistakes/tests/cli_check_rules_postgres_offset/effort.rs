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
    // The closed build() helper now proves its OFFSET; only genuine residuals remain.
    let residuals = advise["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["file"].as_str().unwrap(), row["line"].as_u64().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(
        residuals,
        [("src/query.ts", 11), ("src/query.ts", 12)],
        "{advise}"
    );
    assert!(
        advise["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["file"] == "src/query.ts"
                && row["line"] == 8
                && row["target"].as_str().unwrap().starts_with("offset")),
        "{advise}"
    );
    assert_eq!(
        advise["rules"],
        report(".no-mistakes.low-ignore.yml", None)["rules"]
    );
}
