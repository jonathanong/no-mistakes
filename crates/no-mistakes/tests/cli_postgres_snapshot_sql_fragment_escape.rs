use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report, rules_path};
use std::path::PathBuf;

#[test]
fn postgres_sql_fragment_escape_append_chain_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/eslint-snippets/fixture/executor-append-chain"),
    );
    let config = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/configs/append-chain.yml"),
    );
    // The fixture tree is the existing append-chain parity set. Findings are
    // the escape site, and files that only append a fragment stay absent.
    let edges = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/edges"),
    );
    let cases = [
        ("append_chain", root, config.clone()),
        ("edges", edges, config),
    ];
    assert_saved_fixture("postgres_sql_fragment_escape_append_chain", &cases);
}

#[test]
fn postgres_sql_fragment_escape_sibling_graph_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/sibling-graph"),
    );
    let reports = assert_saved_fixture_report(
        "postgres_sql_fragment_escape_sibling_graph",
        &[("sibling_graph", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let rules = reports["sibling_graph"]["report"]["rules"]
        .as_array()
        .expect("rules");
    assert!(
        rules.iter().any(|finding| {
            finding["rule"] == "postgres-sql-fragment-escape"
                && finding["file"] == "extend.ts"
                && finding["sourceLine"].is_string()
        }),
        "{rules:?}"
    );
    assert!(
        rules
            .iter()
            .all(|finding| finding["rule"] == "postgres-sql-fragment-escape"),
        "{rules:?}"
    );
    assert!(
        rules.iter().all(|finding| {
            finding["message"]
                .as_str()
                .is_none_or(|message| !message.contains("WidgetError"))
        }),
        "{rules:?}"
    );
    for finding in rules {
        if finding.get("file").and_then(|file| file.as_str()).is_some() {
            assert!(finding["sourceLine"].is_string(), "{finding}");
        }
    }
}

#[test]
fn postgres_sql_fragment_escape_absence_gap_matches_siblings() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-fragment-escape/fixtures/absence-gap"),
    );
    let reports = assert_saved_fixture_report(
        "postgres_sql_fragment_escape_absence_gap",
        &[
            ("fragment_on", root.clone(), root.join(".no-mistakes.yml")),
            ("fragment_off", root.clone(), root.join("siblings.yml")),
        ],
        &[],
        &[],
    );
    let on = &reports["fragment_on"]["report"];
    let off = &reports["fragment_off"]["report"];
    assert_eq!(sibling_report(on), sibling_report(off));
    let rules = on["rules"].as_array().expect("rules");
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "postgres-sql-fragment-escape"
            && finding["file"] == "relative-escape.ts"
            && finding["target"] == "COLS"
            && finding["sourceLine"] == "COLS.append(extra);"
    }));
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "postgres-sql-fragment-escape"
            && finding["file"] == "bare-escape.ts"
            && finding["target"] == "COLS"
            && finding["sourceLine"] == "COLS.append(extra);"
    }));
    for report in [on, off] {
        let rules = report["rules"].as_array().expect("rules");
        assert!(
            rules.iter().all(|finding| {
                finding["message"]
                    .as_str()
                    .is_none_or(|message| !message.contains("never constructed"))
            }),
            "{rules:?}"
        );
        for finding in rules {
            if finding.get("file").is_some() {
                assert!(finding["sourceLine"].is_string(), "{finding}");
            }
        }
        let incomplete = report["incomplete"].as_array().expect("incomplete");
        assert!(
            incomplete.iter().any(|row| {
                row["consumer"] == "unconstructed-error-class" && row["file"] == "wide.ts"
            }),
            "{incomplete:?}"
        );
    }
}

fn sibling_report(report: &serde_json::Value) -> serde_json::Value {
    let mut copy = report.clone();
    if let Some(rules) = copy["rules"].as_array_mut() {
        rules.retain(|finding| finding["rule"] != "postgres-sql-fragment-escape");
    }
    if let Some(rows) = copy["incomplete"].as_array_mut() {
        rows.retain(|row| row["consumer"] != "postgres-sql-fragment-escape");
    }
    copy
}

#[test]
fn postgres_sql_fragment_escape_import_resolution_snapshot() {
    let root = rules_path(
        "postgres-sql-fragment-escape",
        "fixtures",
        "import-resolution",
    );
    let reports = assert_saved_fixture_report(
        "postgres_sql_fragment_escape_import_resolution",
        &[("imports", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["imports"]["report"];
    let findings = report["rules"].as_array().expect("findings");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding["file"] == "confirmed.ts"));
    let gaps = report["incomplete"].as_array().expect("incomplete");
    assert_eq!(gaps.len(), 1, "{gaps:?}");
    assert_eq!(gaps[0]["file"], "relevant.ts");
    assert!(gaps[0]["message"]
        .as_str()
        .unwrap()
        .contains("`./missing-columns`"));
}

#[test]
fn postgres_sql_fragment_escape_bounds_import_resolution_work() {
    let root = rules_path(
        "postgres-sql-fragment-escape",
        "fixtures",
        "import-resolution",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--jobs",
            "2",
            "--lock-timeout",
            "0",
            "--verbose-timings",
            "check",
            "--root",
        ])
        .arg(root)
        .args(["--format", "json"])
        .output()
        .expect("run fragment check");
    assert_eq!(output.status.code(), Some(1));
    let diagnostics = String::from_utf8(output.stderr).expect("diagnostics");
    assert!(
        diagnostics
            .lines()
            .any(|line| line == "[work] graph.builds: 1"),
        "{diagnostics}"
    );
    let candidates = diagnostics
        .lines()
        .find_map(|line| {
            line.strip_prefix("[work] postgres_sql_fragment_escape.module_candidates: ")
        })
        .expect("module candidate work")
        .parse::<usize>()
        .expect("candidate count");
    assert!((1..=6).contains(&candidates), "{diagnostics}");
}

#[test]
fn postgres_sql_fragment_escape_relevance_snapshot() {
    let root = rules_path(
        "postgres-sql-fragment-escape",
        "fixtures",
        "fragment-relevance",
    );
    let reports = assert_saved_fixture_report(
        "postgres_sql_fragment_escape_relevance",
        &[("proof", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["proof"];
    assert_eq!(report["exitCode"], 1);
    assert!(report["report"]["rules"].as_array().unwrap().is_empty());
    let gaps = report["report"]["incomplete"].as_array().expect("gaps");
    let files = gaps
        .iter()
        .map(|gap| gap["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files, ["dynamic.ts", "producer.ts", "reexport.ts"]);
    assert!(gaps
        .iter()
        .all(|gap| gap["message"].as_str().unwrap().contains("`./missing`")));
}

#[test]
fn postgres_sql_fragment_escape_module_gap_candidates_snapshot() {
    let root = rules_path(
        "postgres-sql-fragment-escape",
        "fixtures",
        "gap-module-candidates",
    );
    let reports = assert_saved_fixture_report(
        "postgres_sql_fragment_escape_module_gap_candidates",
        &[("candidates", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["candidates"]["report"];
    let gaps = report["incomplete"].as_array().expect("incomplete");
    assert_eq!(gaps.len(), 1, "{gaps:?}");
    assert_eq!(gaps[0]["file"], "known-local.ts");
    assert!(gaps[0]["message"]
        .as_str()
        .unwrap()
        .contains("`./missing.server`"));
    assert!(report["rules"].as_array().unwrap().iter().any(|row| {
        row["file"] == "packages/local/mapped.ts" && row["target"] == "COLS"
    }));
}

#[test]
fn postgres_sql_fragment_escape_dependency_gaps_snapshot() {
    let root = rules_path("postgres-sql-fragment-escape", "fixtures", "dependency-gaps");
    let reports = assert_saved_fixture_report(
        "postgres_sql_fragment_escape_dependency_gaps",
        &[("reached", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["reached"]["report"];
    let gaps = report["incomplete"].as_array().expect("incomplete");
    let files = gaps
        .iter()
        .map(|gap| gap["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        files,
        ["created-broken.ts", "query-broken.ts", "transitive-broken.ts"]
    );
    assert!(gaps.iter().all(|gap| gap["message"]
        .as_str()
        .unwrap()
        .contains("has a fatal parse error")));
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0]["file"], "fragment.ts");
}
