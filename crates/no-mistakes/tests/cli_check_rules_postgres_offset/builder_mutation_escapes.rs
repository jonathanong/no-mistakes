use std::path::PathBuf;
use std::process::Command;

#[test]
fn builder_mutations_cannot_publish_stale_complete_sql() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/builder-mutation-escapes");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
    assert_eq!(output.status.code(), Some(1), "{report}");
    let mut expected = Vec::new();
    for file in [
        "aliases.ts",
        "untyped-helper.ts",
        "container.ts",
        "controls.ts",
        "literal-escapes.ts",
        "reference-controls.ts",
        "forward-escapes.ts",
    ] {
        let source = std::fs::read_to_string(root.join("src").join(file)).unwrap();
        for (index, line) in source.lines().enumerate() {
            let Some((_, kind)) = line.split_once("// expect: ") else {
                continue;
            };
            for rule in ["postgres-no-offset", "postgres-required-predicates"] {
                if kind == "dynamic" || rule == "postgres-no-offset" {
                    expected.push((
                        rule.to_string(),
                        format!("src/{file}"),
                        index as u64 + 1,
                        if kind == "dynamic" {
                            "unanalyzable"
                        } else {
                            "offset"
                        }
                        .to_string(),
                    ));
                }
            }
        }
    }
    let mut actual = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["rule"].as_str().unwrap().to_string(),
                finding["file"].as_str().unwrap().to_string(),
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected, "{report}");
}
