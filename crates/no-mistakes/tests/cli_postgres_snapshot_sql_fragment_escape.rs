use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report};
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
    assert!(rules
        .iter()
        .all(|finding| finding["file"] != "bare-escape.ts"));
    let incomplete = on["incomplete"].as_array().expect("incomplete");
    assert!(incomplete.iter().any(|gap| {
        gap["consumer"] == "postgres-sql-fragment-escape"
            && gap["file"] == "bare-escape.ts"
            && gap["message"]
                .as_str()
                .is_some_and(|message| message.contains("`@example/columns`"))
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
