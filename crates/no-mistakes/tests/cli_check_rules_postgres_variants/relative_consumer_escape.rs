use super::{check, check_with_suppressed, PathBuf};
use std::process::Command;

#[test]
fn rejected_relative_owners_cannot_leave_stale_finite_builder_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/relative-consumer-escape");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// findings: ")
                .map(|(_, target)| (index as u64 + 1, target))
        })
        .collect::<Vec<_>>();
    let report = check("postgres-no-offset", "relative-consumer-escape");
    let findings = report["rules"].as_array().unwrap();
    let actual = findings
        .iter()
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap(),
                finding["target"]
                    .as_str()
                    .unwrap()
                    .split('#')
                    .next()
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{report}");

    let included = check_with_suppressed("postgres-no-offset", "relative-consumer-escape", true);
    let suppressed_line = source
        .lines()
        .position(|line| line.contains("// suppressed: unanalyzable"))
        .unwrap() as u64
        + 1;
    let suppressed = included["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{included}");
    assert_eq!(suppressed[0]["line"], suppressed_line, "{included}");
    assert_eq!(suppressed[0]["rule"], "postgres-no-offset", "{included}");
    assert_eq!(suppressed[0]["directive"]["kind"], "nextLine", "{included}");
    assert_eq!(included["rules"], report["rules"], "{included}");

    // Dynamic reuse is ignored, but accepted pending captures retain their
    // concrete offsets after all arguments have executed.
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("ignore.yml"))
        .output()
        .unwrap();
    let ignored: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let actual = ignored["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap(),
                finding["target"]
                    .as_str()
                    .unwrap()
                    .split('#')
                    .next()
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let expected = expected
        .into_iter()
        .filter(|(_, target)| *target == "offset")
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{ignored}");
}
