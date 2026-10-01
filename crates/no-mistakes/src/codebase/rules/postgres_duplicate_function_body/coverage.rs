use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn messages(file: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/duplicate-function-body");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(&format!("schemaCatalogPath: {file}")).unwrap(),
        ..Default::default()
    });
    check_with_files(&root, &config, &[root.join(file)])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>()
        .join("\n")
}

fn settings(identifiers: bool) -> normalize::Settings {
    normalize::Settings {
        normalize_identifiers: identifiers,
        normalize_raise: true,
        keep_identifiers: Vec::new(),
    }
}

#[test]
fn parameter_names_do_not_set_volatility() {
    let text = messages("modes.json");
    assert!(text.contains("fn_param"), "{text}");
    assert!(text.contains("fn_param_copy"), "{text}");
    assert!(!text.contains("fn_real"), "{text}");
    assert!(!text.contains("fn_c"), "{text}");
}

#[test]
fn unicode_return_types_and_search_paths_stay_distinct() {
    let text = messages("unicode.json");
    assert!(text.contains("fn_alpha"), "{text}");
    assert!(text.contains("fn_alpha_copy"), "{text}");
    assert!(!text.contains("fn_beta"), "{text}");
    assert!(text.contains("fn_cafe"), "{text}");
    assert!(text.contains("fn_cafe_copy"), "{text}");
    assert!(!text.contains("fn_public"), "{text}");
}

#[test]
fn search_path_stops_at_volatility_and_skips_noise() {
    let path = search_path::extract(
        "CREATE FUNCTION f() SET work_mem TO x SET search_path TO /* nest /* in */ */ tenant_a -- note\n, \"ten\"\"ant\", $1$ IMMUTABLE",
    );
    assert!(path.contains("tenant_a"), "{path}");
    assert!(path.contains("\"ten\"\"ant\""), "{path}");
    assert!(!path.contains("immutable"), "{path}");
    assert!(!search_path::extract("SET search_path TO tenant_a STABLE").contains("stable"));
    assert!(!search_path::extract("SET search_path TO tenant_a VOLATILE").contains("volatile"));
    let stopped = search_path::extract("SET search_path TO tenant_a; LANGUAGE sql");
    assert!(!stopped.contains("language"), "{stopped}");
    let commented = search_path::extract("SET -- note\n search_path TO public");
    assert!(commented.contains("public"), "{commented}");
    let unicode = search_path::extract("SET search_path TO café, public");
    assert!(unicode.contains("café"), "{unicode}");
    assert!(unicode.contains("public"), "{unicode}");
}

#[test]
fn clause_boundaries_keep_distinct_functions_apart() {
    let text = messages("boundaries.json");
    assert!(text.contains("fn_set"), "{text}");
    assert!(!text.contains("fn_real_strict"), "{text}");
    assert!(text.contains("fn_text"), "{text}");
    assert!(!text.contains("fn_int"), "{text}");
    assert!(text.contains("fn_send"), "{text}");
    assert!(!text.contains("fn_delete"), "{text}");
    assert!(text.contains("fn_atomic"), "{text}");
    assert!(!text.contains("fn_string"), "{text}");
    assert!(text.contains("fn_u"), "{text}");
}

#[test]
fn table_columns_planner_estimates_and_operators_stay_distinct() {
    let text = messages("planner.json");
    assert!(text.contains("fn_table_int"), "{text}");
    assert!(!text.contains("fn_table_text"), "{text}");
    assert!(text.contains("fn_rows_one"), "{text}");
    assert!(!text.contains("fn_rows_many"), "{text}");
    assert!(text.contains("fn_op_a"), "{text}");
    assert!(!text.contains("fn_op_b"), "{text}");
    assert!(text.contains("fn_support_one"), "{text}");
    assert!(!text.contains("fn_support_two"), "{text}");
    assert!(text.contains("fn_bang"), "{text}");
    assert!(!text.contains("fn_cang"), "{text}");
}

#[test]
fn qualified_raise_calls_stay_calls() {
    let tokens = normalize::normalized_tokens(
        "PERFORM public.raise(send_email())",
        &settings(true),
        Some("plpgsql"),
    )
    .unwrap();
    assert!(tokens.iter().any(|token| token == "raise"), "{tokens:?}");
    assert!(!tokens.iter().any(|token| token == "RAISE"), "{tokens:?}");
}

#[test]
fn normalization_covers_closed_calls_and_quoted_names() {
    let nested = normalize::normalized_tokens(
        "SELECT helper(g(1), (widget) := 2)",
        &settings(true),
        Some("sql"),
    )
    .unwrap();
    assert!(nested.iter().any(|token| token == "helper"));
    let quoted =
        normalize::normalized_tokens("SELECT \"widget\"", &settings(true), Some("sql")).unwrap();
    assert!(quoted.iter().any(|token| token.starts_with("ID")));
}

#[test]
fn outside_body_keeps_text_when_the_span_is_unusable() {
    assert_eq!(scan::outside_body("abcdef", None), "abcdef");
    assert_eq!(scan::outside_body("abcdef", Some((4, 1))), "abcdef");
}
