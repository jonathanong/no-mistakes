use super::*;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/switch-inert-generators")
}

#[test]
fn inert_generator_labels_pass_and_executed_effects_follow_fail_ignore_and_suppression() {
    let source = std::fs::read_to_string(fixture_root().join("input.ts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match line.split_once("// ") {
            Some((_, target @ ("unanalyzable" | "offset"))) => Some((index as u64 + 1, target)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 40);
    let report = check_with_suppressed("postgres-no-offset", "switch-inert-generators", true);
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), expected.len(), "{report}");
    for (finding, (line, target)) in findings.iter().zip(expected) {
        assert_eq!(finding["line"], line, "{report}");
        assert_eq!(finding["target"], target, "{report}");
    }
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{report}");
    let suppressed_line = source
        .lines()
        .position(|line| line.ends_with("// suppressed"))
        .unwrap() as u64
        + 1;
    assert_eq!(suppressed[0]["line"], suppressed_line, "{report}");

    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(fixture_root())
        .arg("--config")
        .arg(fixture_root().join("ignore.yml"))
        .output()
        .unwrap();
    let ignored: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
    let findings = ignored["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{ignored}");
    assert_eq!(findings[0]["target"], "offset", "{ignored}");
    assert_eq!(
        findings[0]["line"],
        source
            .lines()
            .position(|line| line.ends_with("// offset"))
            .unwrap() as u64
            + 1,
        "{ignored}"
    );
}
