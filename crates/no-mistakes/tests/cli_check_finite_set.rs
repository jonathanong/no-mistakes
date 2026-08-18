use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn finite_set_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency")
            .join(name),
    )
}

#[test]
fn finite_set_path_regex_min_size_fails_closed_for_empty_captures() {
    let root = finite_set_fixture("path-regex-empty-minsize");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    let json: serde_json::Value = serde_json::from_str(&body).expect("stdout should be json");
    let rules = json["rules"].as_array().unwrap();

    assert!(!out.status.success(), "{body}");
    assert_eq!(
        rules
            .iter()
            .filter(|finding| finding["rule"] == "finite-set-consistency")
            .count(),
        2,
        "{body}"
    );
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["message"].as_str().is_some_and(|message| {
                message.contains("missingFiles") && message.contains("minSize is 1")
            })
    }));
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "finite-set-consistency"
            && finding["message"].as_str().is_some_and(|message| {
                message.contains("missingLinks") && message.contains("minSize is 1")
            })
    }));
    assert!(rules.iter().all(|finding| {
        finding["rule"] != "finite-set-consistency"
            || !finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("contains"))
    }));
}
