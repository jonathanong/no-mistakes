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
        Some("plpgsql"),
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
        "function:fn_reject_orders_update: function body duplicates 2 other function(s) after normalizing names and literals: fn_reject_invoices_update, fn_reject_refunds_update. Copied functions drift, so a fix has to be repeated in each copy; replace them with one function parameterized by TG_TABLE_NAME / TG_ARGV",
        "function:fn_reject_invoices_update: function body duplicates 2 other function(s) after normalizing names and literals: fn_reject_orders_update, fn_reject_refunds_update. Copied functions drift, so a fix has to be repeated in each copy; replace them with one function parameterized by TG_TABLE_NAME / TG_ARGV",
        "function:fn_reject_refunds_update: function body duplicates 2 other function(s) after normalizing names and literals: fn_reject_invoices_update, fn_reject_orders_update. Copied functions drift, so a fix has to be repeated in each copy; replace them with one function parameterized by TG_TABLE_NAME / TG_ARGV",
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
        "function:fn_01: function body duplicates 6 other function(s) after normalizing names and literals: fn_02, fn_03, fn_04, fn_05, fn_06 and 1 more. Copied functions drift, so a fix has to be repeated in each copy; replace them with one function that takes the varying values as arguments"
    ));
    assert_eq!(joined.matches("function:fn_").count(), 7, "{joined}");
}

#[test]
fn normaliser_covers_literals_calls_levels_and_kept_words() {
    assert_eq!(
        tokens("BEGIN PERFORM fn_a(); RETURN NEW; END", true, true, &[]),
        ["BEGIN", "ID1", "fn_a", "(", ")", ";", "RETURN", "NEW", ";", "END"]
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
        ["RETURN", "NEW", ".", "ID1"]
    );
    assert_eq!(
        tokens("RETURN NEW.\"Body\"", false, true, &[]),
        ["RETURN", "NEW", ".", "\"Body\""]
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

#[test]
fn names_keep_qualifiers_quotes_and_bare_raise() {
    assert_ne!(
        tokens("PERFORM accounting.refresh()", true, true, &[]),
        tokens("PERFORM reporting.refresh()", true, true, &[])
    );
    assert_eq!(
        tokens("SELECT Do_Work()", true, true, &[]),
        tokens("SELECT do_work()", true, true, &[])
    );
    assert_ne!(
        tokens("SELECT \"Do_Work\"()", true, true, &[]),
        tokens("SELECT do_work()", true, true, &[])
    );
    assert_ne!(
        tokens("SELECT MixedCase", false, true, &[]),
        tokens("SELECT \"MixedCase\"", false, true, &[])
    );
    assert_eq!(
        tokens("SELECT MixedCase", false, true, &[]),
        tokens("SELECT mixedcase", false, true, &[])
    );
    assert_ne!(
        tokens("BEGIN RAISE; END", true, true, &[]),
        tokens("BEGIN RAISE EXCEPTION 'x'; END", true, true, &[])
    );
    assert_ne!(
        normalize::normalized_tokens("SELECT raise;", &plain_settings(), Some("sql")),
        normalize::normalized_tokens("SELECT raise + 1;", &plain_settings(), Some("sql"))
    );
    assert_ne!(
        tokens("BEGIN a := 1; b := 2; RETURN a - b; END", true, true, &[]),
        tokens("BEGIN a := 1; b := 2; RETURN b - a; END", true, true, &[])
    );
    assert_eq!(
        tokens("BEGIN a := 1; b := 2; RETURN a - b; END", true, true, &[]),
        tokens("BEGIN x := 1; y := 2; RETURN x - y; END", true, true, &[])
    );
}

#[test]
fn trigger_event_and_ordinary_functions_stay_in_separate_groups() {
    let joined = messages("schemaCatalogPath: kinds.json\n").join("\n");
    assert!(joined.contains("function:fn_event_a:"));
    assert!(joined.contains("event trigger function"));
    assert!(!joined.contains("function:fn_void_same:"));
    assert!(!joined.contains("function:fn_row_trigger:"));
}

#[test]
fn single_quoted_bodies_are_compared() {
    let joined = messages("schemaCatalogPath: quoted.json\n").join("\n");
    assert!(joined.contains("function:fn_quote_a:"), "{joined}");
    assert!(joined.contains("function:fn_quote_b:"));
}

#[test]
fn a_default_string_does_not_make_an_event_trigger() {
    let joined = messages("schemaCatalogPath: event-default.json\n").join("\n");
    assert!(joined.is_empty(), "{joined}");
}

#[test]
fn a_body_that_repeats_the_language_name_keeps_security_definer() {
    let joined = messages("schemaCatalogPath: security-body.json\n").join("\n");
    assert!(joined.contains("function:fn_definer_sql:"), "{joined}");
    assert!(joined.contains("function:fn_definer_sql_copy:"));
    assert!(!joined.contains("function:fn_invoker_sql:"));
}

#[test]
fn volatility_return_contract_and_named_arguments_split_copies() {
    let volatility = messages("schemaCatalogPath: volatility.json\n").join("\n");
    let immutable = volatility
        .lines()
        .find(|line| line.contains("function:fn_immutable:"))
        .unwrap_or("");
    assert!(immutable.contains("fn_immutable_copy"), "{volatility}");
    assert!(!immutable.contains("fn_volatile"), "{volatility}");

    let returns = messages("schemaCatalogPath: returns.json\n").join("\n");
    let integer = returns
        .lines()
        .find(|line| line.contains("function:fn_int:"))
        .unwrap_or("");
    assert!(integer.contains("fn_int_copy"), "{returns}");
    assert!(!integer.contains("fn_bigint"), "{returns}");
    assert!(!returns.contains("function:fn_bigint:"), "{returns}");

    assert_ne!(
        tokens("SELECT target(left_value => 1)", true, true, &[]),
        tokens("SELECT target(right_value => 1)", true, true, &[])
    );
    assert_ne!(
        tokens("SELECT target(left_value := 1)", true, true, &[]),
        tokens("SELECT target(right_value := 1)", true, true, &[])
    );
    let named = messages("schemaCatalogPath: named.json\n").join("\n");
    let left = named
        .lines()
        .find(|line| line.contains("function:fn_left:"))
        .unwrap_or("");
    assert!(left.contains("fn_left_copy"), "{named}");
    assert!(!left.contains("fn_right"), "{named}");
}

#[test]
fn comments_quoted_paths_concat_and_return_expressions() {
    assert!(
        messages("schemaCatalogPath: comment-security.json\n").is_empty(),
        "{}",
        messages("schemaCatalogPath: comment-security.json\n").join("\n")
    );
    let path = messages("schemaCatalogPath: quoted-path.json\n").join("\n");
    let quoted = path
        .lines()
        .find(|line| line.contains("function:fn_quoted_a:"))
        .unwrap_or("");
    assert!(quoted.contains("fn_quoted_a_copy"), "{path}");
    assert!(!quoted.contains("fn_quoted_lower"), "{path}");
    assert!(
        messages("schemaCatalogPath: concat.json\n").is_empty(),
        "{}",
        messages("schemaCatalogPath: concat.json\n").join("\n")
    );
    let returned = messages("schemaCatalogPath: return-expr.json\n").join("\n");
    assert!(returned.contains("function:fn_ret_a:"), "{returned}");
    assert!(returned.contains("function:fn_ret_b:"));
}

#[test]
fn escape_strings_and_atomic_language_are_compared() {
    let escaped = messages("schemaCatalogPath: escape.json\n").join("\n");
    assert!(escaped.contains("function:fn_escape_a:"), "{escaped}");
    assert!(escaped.contains("function:fn_escape_b:"));
    let atomic = messages("schemaCatalogPath: atomic.json\n").join("\n");
    assert!(atomic.contains("function:fn_atomic_a:"), "{atomic}");
    assert!(atomic.contains("function:fn_atomic_b:"));
}

#[test]
fn strict_functions_are_not_the_same_copy_as_called_on_null() {
    let joined = messages("schemaCatalogPath: strict.json\n").join("\n");
    let strict = joined
        .lines()
        .find(|line| line.contains("function:fn_strict:"))
        .unwrap_or("");
    assert!(strict.contains("fn_strict_long"), "{joined}");
    assert!(!strict.contains("fn_called"), "{joined}");
    assert!(!strict.contains("fn_default_word"), "{joined}");
    assert!(joined.contains("function:fn_called:"));
    assert!(joined.contains("function:fn_default_word:"));
}

#[test]
fn security_definer_is_not_the_same_copy_as_invoker() {
    let joined = messages("schemaCatalogPath: security.json\n").join("\n");
    assert!(joined.contains("function:fn_definer:"), "{joined}");
    assert!(joined.contains("function:fn_definer_copy:"));
    assert!(!joined.contains("function:fn_invoker:"));
}

#[test]
fn header_details_keep_distinct_functions_apart() {
    let paths = messages("schemaCatalogPath: newline-path.json\n").join("\n");
    assert!(paths.contains("function:fn_tenant_a:"), "{paths}");
    assert!(!paths.contains("function:fn_tenant_b:"));
    assert!(messages("schemaCatalogPath: quoted-return.json\n").is_empty());
    assert!(messages("schemaCatalogPath: hex-escape.json\n").is_empty());
    let kept = messages("schemaCatalogPath: kept-name.json\nkeepIdentifiers:\n  - MixedCase\n");
    assert!(kept.is_empty(), "{kept:?}");
    let leak = messages("schemaCatalogPath: leakproof.json\n").join("\n");
    assert!(leak.contains("function:fn_plain:"), "{leak}");
    assert!(!leak.contains("function:fn_leak:"));
}

#[test]
fn array_parallel_and_escape_defaults_stay_apart() {
    let arrays = messages("schemaCatalogPath: arrays.json\n").join("\n");
    assert!(arrays.contains("function:fn_int:"), "{arrays}");
    assert!(!arrays.contains("function:fn_array:"));
    let parallel = messages("schemaCatalogPath: parallel.json\n").join("\n");
    assert!(parallel.contains("function:fn_safe:"), "{parallel}");
    assert!(!parallel.contains("function:fn_unsafe:"));
    let escaped = messages("schemaCatalogPath: escape-default.json\n").join("\n");
    assert!(escaped.is_empty(), "{escaped}");
}

#[test]
fn different_search_paths_are_not_the_same_copy() {
    let joined = messages("schemaCatalogPath: search-path.json\n").join("\n");
    assert!(joined.contains("function:fn_tenant_a:"), "{joined}");
    assert!(joined.contains("function:fn_tenant_a_copy:"));
    assert!(!joined.contains("function:fn_tenant_b:"));
}

fn plain_settings() -> normalize::Settings {
    normalize::Settings {
        normalize_identifiers: true,
        normalize_raise: true,
        keep_identifiers: Vec::new(),
    }
}

#[test]
fn include_and_custom_message_apply_to_the_catalog() {
    let mut compiled = config("schemaCatalogPath: rejects.json\n");
    compiled.rules[0].include = vec!["services/api/**".to_string()];
    let root = fixture();
    let findings = check_with_files(&root, &compiled, &[root.join("rejects.json")]).unwrap();
    assert!(findings.is_empty());

    let mut compiled = config("schemaCatalogPath: many.json\n");
    compiled.rules[0].message = Some("merge the copies".to_string());
    let joined = check_with_files(&root, &compiled, &[root.join("many.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("merge the copies"));
    assert!(!joined.contains("TG_TABLE_NAME"));
}
