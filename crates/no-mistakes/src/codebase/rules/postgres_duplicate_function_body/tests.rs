use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/duplicate-function-body")
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn messages(options: &str) -> Vec<String> {
    let root = fixture();
    let path = options
        .lines()
        .find_map(|line| line.strip_prefix("schemaCatalogPath: "))
        .unwrap_or("schema.json");
    check_with_files(&root, &config(options), &[root.join(path)])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn tokens(body: &str, identifiers: bool, raise: bool, keep: &[&str]) -> Vec<String> {
    normalize::normalized_tokens(
        body,
        &normalize::Settings {
            normalize_identifiers: identifiers,
            normalize_raise: raise,
            keep_identifiers: keep.iter().map(|word| word.to_ascii_uppercase()).collect(),
        },
    )
    .unwrap_or_default()
}

const RAISE: &[&str] = &["BEGIN", "RAISE", "EXCEPTION", "?", ";", "END"];

#[test]
fn copied_raise_functions_normalize_to_one_body_and_report_all_three() {
    assert_eq!(
        tokens(
            "BEGIN RAISE EXCEPTION 'orders rows are immutable' USING ERRCODE = '23514'; END",
            true,
            true,
            &[],
        ),
        RAISE
    );
    assert_eq!(
        tokens(
            "BEGIN RAISE EXCEPTION 'invoices rows are immutable'; END;",
            true,
            true,
            &[]
        ),
        RAISE
    );
    assert_eq!(
        tokens(
            "BEGIN\n  RAISE EXCEPTION '% rows are immutable', TG_TABLE_NAME USING ERRCODE = 'check_violation';\nEND;",
            true,
            true,
            &[],
        ),
        RAISE
    );
    let joined = messages("schemaCatalogPath: rejects.json\n").join("\n");
    assert_eq!(joined.matches("function:").count(), 3, "{joined}");
    for text in [
        "function:fn_reject_orders_update: function body duplicates 2 other function(s) after normalising names and literals: fn_reject_invoices_update, fn_reject_refunds_update; replace them with one function parameterised by TG_TABLE_NAME / TG_ARGV",
        "function:fn_reject_invoices_update: function body duplicates 2 other function(s) after normalising names and literals: fn_reject_orders_update, fn_reject_refunds_update; replace them with one function parameterised by TG_TABLE_NAME / TG_ARGV",
        "function:fn_reject_refunds_update: function body duplicates 2 other function(s) after normalising names and literals: fn_reject_invoices_update, fn_reject_orders_update; replace them with one function parameterised by TG_TABLE_NAME / TG_ARGV",
    ] {
        assert!(joined.contains(text), "missing {text}\n{joined}");
    }
}

#[test]
fn valid_shapes_are_not_reported() {
    let joined = messages("schemaCatalogPath: schema.json\n").join("\n");
    for absent in [
        "fn_touch_orders",
        "fn_touch_invoices",
        "fn_sql_copy",
        "fn_plpgsql_copy",
        "fn_reject_mutation",
        "fn_flow_if",
        "fn_flow_case",
        "fn_bad",
        "fn_nobody",
    ] {
        assert!(!joined.contains(absent), "{absent} in {joined}");
    }
    assert!(joined.contains("function:fn_title:"));
    assert!(joined.contains("function:fn_body:"));
    assert!(joined.contains("function:fn_semi_a:"));
    assert!(joined.contains("function:fn_semi_b:"));
    assert!(joined.contains("function:fn_audit(integer):"));
    assert!(joined.contains("function:fn_audit(text):"));
    assert!(joined.contains("function:fn_dollar_a:"));
    assert!(joined.contains("function:fn_dollar_b:"));
    assert!(joined.contains("function:fn_nolang_a:"));
    assert!(joined.contains("function:fn_nolang_b:"));
}

#[test]
fn min_tokens_and_disabled_normalisers_split_copies() {
    assert!(messages("schemaCatalogPath: rejects.json\nminTokens: 12\n").is_empty());
    assert!(messages("schemaCatalogPath: rejects.json\nnormalizeRaise: false\n").is_empty());
    let raw = messages("schemaCatalogPath: schema.json\nnormalizeIdentifiers: false\n").join("\n");
    assert!(!raw.contains("function:fn_title:"));
    assert!(!raw.contains("function:fn_body:"));
    assert!(raw.contains("function:fn_semi_a:"));
}

#[test]
fn clusters_above_five_list_the_rest_as_and_more() {
    let joined = messages("schemaCatalogPath: many.json\n").join("\n");
    assert!(joined.contains(
        "function:fn_01: function body duplicates 6 other function(s) after normalising names and literals: fn_02, fn_03, fn_04, fn_05, fn_06 and 1 more; replace them with one function parameterised by TG_TABLE_NAME / TG_ARGV"
    ));
    assert_eq!(joined.matches("function:fn_").count(), 7, "{joined}");
}

#[test]
fn normaliser_covers_literals_calls_levels_and_kept_words() {
    assert_eq!(
        tokens("BEGIN PERFORM fn_a(); RETURN NEW; END", true, true, &[]),
        ["BEGIN", "ID", "fn_a", "(", ")", ";", "RETURN", "NEW", ";", "END"]
    );
    assert_ne!(
        tokens("BEGIN PERFORM fn_a(); END", true, true, &[]),
        tokens("BEGIN PERFORM fn_b(); END", true, true, &[])
    );
    assert_eq!(
        tokens("RETURN 'a'", true, true, &[]),
        tokens("RETURN $$b$$", true, true, &[])
    );
    assert_eq!(tokens("RETURN $$b$$", true, true, &[]), ["RETURN", "'?'"]);
    assert_eq!(
        tokens("RETURN $tag$c$tag$", true, true, &[]),
        ["RETURN", "'?'"]
    );
    assert_eq!(tokens("RETURN 1", true, true, &[]), ["RETURN", "0"]);
    assert_eq!(tokens("RETURN 2.5", true, true, &[]), ["RETURN", "0"]);
    assert_eq!(
        tokens("RETURN NEW.title", true, true, &[]),
        ["RETURN", "NEW", ".", "ID"]
    );
    assert_eq!(
        tokens("RETURN NEW.\"Body\"", false, true, &[]),
        ["RETURN", "NEW", ".", "Body"]
    );
    assert_eq!(
        tokens("BEGIN RETURN now; END", true, true, &["now"]),
        ["BEGIN", "RETURN", "NOW", ";", "END"]
    );
    assert_ne!(
        tokens("BEGIN RETURN now; END", true, true, &["now"]),
        tokens("BEGIN RETURN today; END", true, true, &["now"])
    );
    assert_eq!(
        tokens(
            "IF TG_TABLE_NAME IS NULL THEN RETURN NEW; END IF",
            true,
            true,
            &[]
        ),
        [
            "IF",
            "TG_TABLE_NAME",
            "IS",
            "NULL",
            "THEN",
            "RETURN",
            "NEW",
            ";",
            "END",
            "IF"
        ]
    );
    for level in ["WARNING", "NOTICE", "INFO", "LOG", "DEBUG", "EXCEPTION"] {
        let body = format!("BEGIN RAISE {level} 'x' USING ERRCODE = '1', format('%s', 1); END");
        assert_eq!(
            tokens(&body, true, true, &[]),
            ["BEGIN", "RAISE", level, "?", ";", "END"]
        );
    }
    assert_eq!(
        tokens("BEGIN RAISE 'plain'; END", true, true, &[]),
        ["BEGIN", "RAISE", "EXCEPTION", "?", ";", "END"]
    );
    assert_ne!(
        tokens("BEGIN RAISE NOTICE 'x'; END", true, false, &[]),
        tokens("BEGIN RAISE EXCEPTION 'x'; END", true, false, &[])
    );
    assert_eq!(
        tokens(
            "BEGIN RAISE EXCEPTION 'a; b'; RETURN 1; END",
            true,
            true,
            &[]
        ),
        [
            "BEGIN",
            "RAISE",
            "EXCEPTION",
            "?",
            ";",
            "RETURN",
            "0",
            ";",
            "END"
        ]
    );
    assert!(tokens("BEGIN RAISE EXCEPTION 'unterminated", true, true, &[]).is_empty());
    assert_eq!(
        tokens("BEGIN RETURN NEW; END;", true, true, &[]),
        tokens("BEGIN RETURN NEW; END", true, true, &[])
    );
}

#[test]
fn repeated_runs_are_identical() {
    let options = "schemaCatalogPath: rejects.json\n";
    assert_eq!(messages(options), messages(options));
}

#[test]
fn allow_suppresses_one_function_and_reports_stale_entries() {
    let options = "schemaCatalogPath: rejects.json\nallow:\n  - object: function:fn_reject_orders_update\n    reason: Kept separate on purpose\n  - object: function:fn_orders_audit\n    reason: not a finding\n";
    let joined = messages(options).join("\n");
    assert!(!joined.contains("function:fn_reject_orders_update:"));
    assert!(joined.contains("function:fn_reject_invoices_update:"));
    assert!(joined
        .contains("stale postgres-duplicate-function-body allow entry: function:fn_orders_audit"));
}

#[test]
fn option_errors_name_the_field() {
    let cases = [
        ("minClusterSize: 2\n", "option schemaCatalogPath: required"),
        (
            "schemaCatalogPath: ' '\n",
            "option schemaCatalogPath: required",
        ),
        (
            "schemaCatalogPath: rejects.json\nminClusterSize: 1\n",
            "option minClusterSize: must be at least 2",
        ),
        (
            "schemaCatalogPath: rejects.json\nminTokens: 0\n",
            "option minTokens: must be at least 1",
        ),
        (
            "schemaCatalogPath: rejects.json\nallow:\n  - object: function:fn_reject_orders_update\n    reason: ' '\n",
            "option allow: entry function:fn_reject_orders_update needs a reason",
        ),
        (
            "schemaCatalogPath: rejects.json\nallow:\n  - object: nope\n    reason: because\n",
            "option allow: invalid object ref nope",
        ),
        (
            "schemaCatalogPath: rejects.json\nallow:\n  - object: function:fn_reject_orders_update\n    reason: one\n  - object: function:fn_reject_orders_update\n    reason: two\n",
            "option allow: duplicate entry function:fn_reject_orders_update",
        ),
    ];
    for (options, expected) in cases {
        let error = check_with_files(&fixture(), &config(options), &[]).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "{expected} not in {error}"
        );
    }
}
