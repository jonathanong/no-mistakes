use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use super::saved_fixture::materialize_path;
use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "ast-grep-ignore-freshness";

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_all(source: &Path) -> tempfile::TempDir {
    let dir = materialize_path(source);
    git(dir.path(), &["init", "-q", "--initial-branch=main"]);
    git(dir.path(), &["add", "-A"]);
    dir
}

fn tracked_repo() -> tempfile::TempDir {
    let dir = materialize_path(&rules_path(RULE, "fixture", "tracked"));
    let root = dir.path();
    git(root, &["init", "-q", "--initial-branch=main"]);
    git(
        root,
        &[
            "add",
            "--",
            ".no-mistakes.yml",
            "sgconfig.yml",
            "rules",
            "src",
            ".widgets",
            "fixtures",
            "tracked-ignored",
            "custom",
            "fresh",
        ],
    );
    std::fs::rename(
        root.join("gitignore-after.fixture"),
        root.join(".gitignore"),
    )
    .unwrap();
    git(root, &["add", "--", ".gitignore"]);
    dir
}

fn targets(report: &serde_json::Value) -> Vec<String> {
    report["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            format!(
                "{}:{}",
                finding["file"].as_str().unwrap_or(""),
                finding["target"].as_str().unwrap_or("")
            )
        })
        .collect()
}

#[test]
fn ast_grep_ignore_freshness_json_matches_snapshot() {
    let repo = tracked_repo();
    let root = repo.path().to_path_buf();
    let explicit = git_all(&rules_path(RULE, "fixture", "explicit"));
    let no_sgconfig = git_all(&rules_path(RULE, "fixture", "no-sgconfig"));
    let sgconfig_yaml = git_all(&rules_path(RULE, "fixture", "sgconfig-yaml"));
    let nested = git_all(&rules_path(RULE, "fixture", "nested"));
    let report = assert_saved_fixture_report(
        "ast_grep_ignore_freshness",
        &[
            ("tracked", root, PathBuf::new()),
            (
                "explicit_files",
                explicit.path().to_path_buf(),
                PathBuf::new(),
            ),
            (
                "no_sgconfig",
                no_sgconfig.path().to_path_buf(),
                PathBuf::new(),
            ),
            (
                "sgconfig_yaml",
                sgconfig_yaml.path().to_path_buf(),
                PathBuf::new(),
            ),
            ("nested", nested.path().to_path_buf(), PathBuf::new()),
        ],
        &[],
        &[],
    );
    assert_eq!(report["no_sgconfig"]["exitCode"], 0);
    assert_eq!(report["sgconfig_yaml"]["exitCode"], 0);
    assert_ne!(report["tracked"]["exitCode"], 0);
    let tracked = targets(&report["tracked"]);
    for fresh in [
        "rules/widgets.yml:src/items.ts",
        "rules/widgets.yml:src/orders/**/*.ts",
        "rules/widgets.yml:.widgets/items.ts",
        "rules/widgets.yml:.widgets/**/*.ts",
        "rules/widgets.yml:fixtures/orders/**/*.ts",
        "rules/widgets.yml:tracked-ignored/items.ts",
        "rules/widgets.yml:SRC/ORDERS/**/*.TS",
        "rules/widgets.yml:src/also-missing.ts",
        "rules/skipped.yml:src/retired.ts",
        "custom/widgets.yml:custom/missing.ts",
    ] {
        assert!(
            !tracked.iter().any(|hit| hit == fresh),
            "{fresh} in {tracked:?}"
        );
    }
    for stale in [
        "rules/widgets.yml:src/missing.ts",
        "rules/widgets.yml:./src/items.ts",
        "rules/flow.yml:src/missing.ts",
        "rules/multi.yml:src/missing.ts",
        "rules/negated.yml:!src/items.ts",
        "rules/widgets.yml:retired/**/*.ts",
        "rules/widgets.yml:untracked/items.ts",
        "rules/widgets.yml:ignored/items.ts",
        "rules/widgets.yml:RETIRED/**/*.TS",
        "rules/widgets.yml:[unterminated",
        "rules/listed.yml:listed/missing.ts",
        "rules/scalar.yml:ignores",
    ] {
        assert!(
            tracked.iter().any(|hit| hit == stale),
            "missing {stale} in {tracked:?}"
        );
    }
    assert!(report["explicit_files"]["report"]["advisories"]
        .as_array()
        .is_some_and(Vec::is_empty));
    let explicit_hits = targets(&report["explicit_files"]);
    assert!(explicit_hits
        .iter()
        .any(|hit| hit == "custom/widgets.yml:custom/missing.ts"));
    assert!(explicit_hits
        .iter()
        .any(|hit| hit.starts_with("missing/widgets.yml:")));
    assert!(!explicit_hits
        .iter()
        .any(|hit| hit.contains("src/missing.ts")));
    let flow = report["tracked"]["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| {
            finding["file"] == "rules/flow.yml" && finding["target"] == "src/missing.ts"
        })
        .unwrap();
    assert_eq!(
        flow["sourceLine"],
        "ignores: ['src/missing.ts', 'src/items.ts']"
    );
    assert!(!tracked
        .iter()
        .any(|hit| hit.starts_with("rules/flow-suppressed.yml:")));
    assert!(!tracked
        .iter()
        .any(|hit| hit == "rules/flow.yml:src/items.ts" || hit == "rules/multi.yml:src/items.ts"));
    let nested_hits = targets(&report["nested"]);
    assert!(nested_hits
        .iter()
        .any(|hit| hit == "packages/widgets/rules/orders.yml:src/orders.ts"));
    assert!(nested_hits
        .iter()
        .any(|hit| hit == "packages/widgets/rules/orders.yml:src/missing.ts"));
    assert!(!nested_hits.iter().any(|hit| hit.ends_with(":src/items.ts")));
}
