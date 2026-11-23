use super::{compile_options, in_scope, is_script, Options, Scopes, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::discover_files;
use crate::config::v2::load_v2_config;
use std::path::{Path, PathBuf};

fn saved(kind: &str, name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/test-fixture-size-cap")
            .join(kind)
            .join(name),
    )
}

fn compiled(
    each: &[&str],
    threshold: Option<u64>,
    tests: &[&str],
    helpers: &[&str],
) -> super::Compiled {
    compile_options(&Options {
        threshold,
        exemptions: vec![1024],
        each_calls: each.iter().map(|call| (*call).to_string()).collect(),
        each_table_mode: Default::default(),
        scopes: Scopes {
            tests: tests.iter().map(|glob| (*glob).to_string()).collect(),
            helpers: helpers.iter().map(|glob| (*glob).to_string()).collect(),
        },
    })
    .unwrap()
}

fn standard() -> super::Compiled {
    compiled(
        &["it.each"],
        Some(1000),
        &["*.test.ts", "**/*.test.ts"],
        &["helpers/*.ts", "helpers/**/*.ts"],
    )
}

fn expect_with(source: &str, policy: &super::Compiled, expected: &[&str]) {
    let actual: Vec<_> = super::scan::scan_source("widgets.test.ts", source, policy)
        .into_iter()
        .map(|finding| finding.message)
        .collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|message| (*message).to_string())
        .collect();
    assert_eq!(actual, expected, "{source}");
}

fn expect(source: &str, expected: &[&str]) {
    expect_with(source, &standard(), expected);
}

fn check_saved(root: &str, config: &str) -> anyhow::Result<Vec<RuleFinding>> {
    let root = saved("fixture", root);
    let config = load_v2_config(&root, Some(&saved("configs", config)))?;
    let files = discover_files(&root, &[]);
    super::check_with_files(&root, &config, &files)
}

fn sorted_messages(findings: &[RuleFinding]) -> Vec<&str> {
    let mut messages: Vec<_> = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect();
    messages.sort_unstable();
    messages
}

const OVER_FROM: &str = "Array.from length 1001 exceeds threshold 1000";
const OVER_NEW: &str = "new Array length 10000 exceeds threshold 1000";

#[test]
fn defaults_leave_each_calls_empty_until_configured() {
    let defaults = Options::default();
    assert!(defaults.each_calls.is_empty());
    assert!(defaults.threshold.is_none());
    assert!(defaults.exemptions.is_empty());
    assert!(defaults.scopes.tests.is_empty());
    let partial: Options = serde_yaml::from_str("threshold: 1000\n").unwrap();
    assert!(partial.each_calls.is_empty());
    let listed: Options = serde_yaml::from_str("eachCalls: [\"it.each\"]\n").unwrap();
    assert_eq!(listed.each_calls, ["it.each"]);
    let explicit: Options = serde_yaml::from_str("eachTableMode: per-element\n").unwrap();
    assert!(matches!(
        explicit.each_table_mode,
        super::EachTableMode::PerElement
    ));
}

#[test]
fn option_errors_name_the_bad_field() {
    for call in ["it", ".each", "it.", "a.b.c", "it.each ", ""] {
        let Err(error) = compile_options(&Options {
            each_calls: vec![call.to_string()],
            ..Options::default()
        }) else {
            panic!("expected eachCalls error for {call}");
        };
        let error = error.to_string();
        assert!(error.contains("eachCalls"), "{call}: {error}");
    }
    let Err(error) = compile_options(&Options {
        scopes: Scopes {
            tests: vec!["[".to_string()],
            ..Scopes::default()
        },
        ..Options::default()
    }) else {
        panic!("expected an invalid glob error");
    };
    let error = error.to_string();
    assert!(error.contains("scopes.tests"), "{error}");
}

#[test]
fn scopes_match_configured_globs_only() {
    let policy = standard();
    assert!(in_scope(&policy, "widgets.test.ts"));
    assert!(in_scope(&policy, "suite/items.test.ts"));
    assert!(in_scope(&policy, "helpers/make-widgets.ts"));
    assert!(!in_scope(&policy, "src/orders.ts"));
    assert!(is_script("widgets.test.ts"));
    for extension in [".cjs", ".cts", ".js", ".jsx", ".mjs", ".mts", ".ts", ".tsx"] {
        assert!(is_script(&format!("items{extension}")));
    }
    assert!(!is_script("helpers/notes.md"));
    let nested = compiled(
        &["it.each"],
        Some(1000),
        &["**/*.test.ts"],
        &["helpers/**/*.ts"],
    );
    assert!(in_scope(&nested, "widgets.test.ts"), "**/*.test.ts");
    assert!(
        in_scope(&nested, "helpers/make-widgets.ts"),
        "helpers/**/*.ts"
    );
}

#[test]
fn scan_file_skips_missing_out_of_scope_and_non_scripts() {
    let policy = standard();
    let sources = super::super::source_store_for_files(&[]);
    let root = Path::new("/tmp/widgets");
    assert!(
        super::scan::scan_file(root, &root.join("widgets.test.ts"), &policy, &sources).is_empty()
    );
    assert!(
        super::scan::scan_file(root, &root.join("src/orders.ts"), &policy, &sources).is_empty()
    );
    assert!(
        super::scan::scan_file(root, &root.join("helpers/notes.md"), &policy, &sources).is_empty()
    );
}

#[test]
fn missing_threshold_and_parse_errors_are_silent() {
    let open = compiled(&["it.each"], None, &["*.test.ts"], &[]);
    expect_with("new Array(5000);", &open, &[]);
    expect("const rows = Array.from({ length: 5_000 }", &[]);
    expect("const =", &[]);
}

#[test]
fn literal_lengths_use_integer_value_threshold_and_exemptions() {
    expect(
        "const rows = Array.from({ length: 1_001 }, () => ({ name: 'widget' }));",
        &[OVER_FROM],
    );
    expect(
        "const boundary = Array.from({ length: 1_000 }, () => 1);",
        &[],
    );
    expect("const width = Array.from({ length: 1_024 }, () => 1);", &[]);
    expect("const width = Array.from({ length: 0x400 }, () => 1);", &[]);
    expect(
        "const boundary = Array.from({ length: 1e3 }, () => 1);",
        &[],
    );
    expect(
        "const rows = Array.from({ length: 1.001e3 }, () => 1);",
        &[OVER_FROM],
    );
    expect("const wide = new Array(10_000);", &[OVER_NEW]);
    expect("const exact = new Array(1000);", &[]);
    expect("const zero = new Array(0);", &[]);
    expect("const listed = new Array(1, 2, 3);", &[]);
    expect("const empty = new Array();", &[]);
    expect("const dynamic = new Array(count);", &[]);
    expect("const negative = new Array(-1);", &[]);
    expect("const fraction = new Array(1.5);", &[]);
    expect("const huge = new Array(1e20);", &[]);
    expect("const infinite = new Array(1e309);", &[]);
    expect("const spread = new Array(...items);", &[]);
    expect("const called = Array(10_000);", &[OVER_NEW]);
    expect("const exact_call = Array(1000);", &[]);
    expect("const listed_call = Array(1, 2, 3);", &[]);
    expect("const empty_call = Array();", &[]);
    expect("const dynamic_call = Array(count);", &[]);
    expect("Array?.(10_000);", &[]);
    expect(
        "const wrapped_call = Array((10001));",
        &["new Array length 10001 exceeds threshold 1000"],
    );
    expect(
        "const wrapped = new Array((1001));",
        &["new Array length 1001 exceeds threshold 1000"],
    );
    expect(
        "const typed = new Array<Widget>(10001);",
        &["new Array length 10001 exceeds threshold 1000"],
    );
    assert_eq!(super::literals::integer_cardinality(-1.0), None);
    assert_eq!(super::literals::integer_cardinality(f64::NAN), None);
    assert_eq!(super::literals::integer_cardinality(f64::INFINITY), None);
    assert_eq!(super::literals::integer_cardinality(1001.0), Some(1001));
}

#[test]
fn array_from_uses_the_last_literal_length_only() {
    expect("Array.from({ length: rows.length });", &[]);
    expect("Array.from();", &[]);
    expect("Array.from(items);", &[]);
    expect("Array.from({});", &[]);
    expect("Items.from({ length: 5000 });", &[]);
    expect("Array.from?.({ length: 5000 });", &[]);
    expect("new window.Array(5000);", &[]);
    expect(
        "const rows = Array.from<Widget>({ length: 1001 }, () => ({ name: 'widget' }));",
        &[OVER_FROM],
    );
    expect(
        "const rows = Array.from({ length: 1001 as number });",
        &[OVER_FROM],
    );
    expect(
        "const rows = Array.from({ \"length\": 1001 });",
        &[OVER_FROM],
    );
    expect(
        "const rows = Array.from({ [\"length\"]: 1001 });",
        &[OVER_FROM],
    );
    expect(
        "const length = 1001; const rows = Array.from({ [length]: 1 });",
        &[],
    );
    expect(
        "const rows = Array.from({ get length() { return 1001; } });",
        &[],
    );
    expect(
        "const rows = Array.from({ ...extra, length: 1001 });",
        &[OVER_FROM],
    );
    expect("const rows = Array.from({ length: 1001, length: 5 });", &[]);
    expect(
        "const count = 1; const rows = Array.from({ length: 1001, length: count });",
        &[],
    );
    expect(
        "const rows = Array.from({ length: 5, length: 1001 });",
        &[OVER_FROM],
    );
    expect(
        "const rows = Array.from({ length: 1001, get length() { return 1; } });",
        &[],
    );
    expect(
        "const rows = Array.from({ length: 1001, set length(value) { stored = value; } });",
        &[],
    );
    expect("const rows = Array.from({ length: 1001, ...extra });", &[]);
    expect("const rows = Array.from({ length: 1001, [key]: 1 });", &[]);
    expect(
        "const rows = Array.from({ length: 1001, [\"name\"]: 1 });",
        &[OVER_FROM],
    );
    expect(
        "const rows = Array.from({ length: 1001, [0]: 1 });",
        &[OVER_FROM],
    );
}

#[test]
fn each_lists_follow_the_configured_callee() {
    expect(
        "it.each([10_000, 100_000])('scale %i', () => {});",
        &[
            "it.each size 10000 exceeds threshold 1000",
            "it.each size 100000 exceeds threshold 1000",
        ],
    );
    expect("it.each([{ port: 5432 }])('record %j', () => {});", &[]);
    expect("it.each([10, 100])('small %i', () => {});", &[]);
    expect("it.each([])('none', () => {});", &[]);
    expect(
        "it.each([10_000,,1])('hole', () => {});",
        &["it.each size 10000 exceeds threshold 1000"],
    );
    expect(
        "it.each([10_000, ...rest])('spread', () => {});",
        &["it.each size 10000 exceeds threshold 1000"],
    );
    expect(
        "it.each([10_000, 'widget', 10_001])('mixed', () => {});",
        &[
            "it.each size 10000 exceeds threshold 1000",
            "it.each size 10001 exceeds threshold 1000",
        ],
    );
    expect(
        "it.each([10_000, [10_001]])('nested', () => {});",
        &["it.each size 10000 exceeds threshold 1000"],
    );
    expect("it.each(items)('other', () => {});", &[]);
    expect("it.each()('missing', () => {});", &[]);
    expect("test.each([50_000])('other %i', () => {});", &[]);
    expect("it.each?.([50000])('scale', () => {});", &[]);
    expect("it?.each([50000])('scale', () => {});", &[]);
    expect("(it).each([50000])('scale', () => {});", &[]);
    let configured = compiled(&["test.each"], Some(1000), &["*.test.ts"], &[]);
    expect_with(
        "test.each([50_000])('scale', () => {});",
        &configured,
        &["test.each size 50000 exceeds threshold 1000"],
    );
    expect_with("it.each([50_000])('scale', () => {});", &configured, &[]);
    let disabled = compiled(&[], Some(1000), &["*.test.ts"], &[]);
    expect_with(
        "it.each([50_000])('scale', () => {}); const rows = Array.from({ length: 1001 }, () => 1);",
        &disabled,
        &[OVER_FROM],
    );
}

#[test]
fn saved_policy_reports_only_in_scope_literals() {
    let findings = check_saved("policy", "policy.yml").unwrap();
    assert_eq!(
        sorted_messages(&findings),
        [
            "Array.from length 1001 exceeds threshold 1000",
            "Array.from length 2000 exceeds threshold 1000",
            OVER_NEW,
        ]
    );
    assert!(findings.iter().all(|finding| finding.rule == RULE_ID));
    assert!(findings.iter().all(|finding| finding.line > 0));
    assert!(findings
        .iter()
        .all(|finding| !finding.file.contains("orders") && !finding.file.ends_with(".md")));
    assert!(check_saved("pass", "policy.yml").unwrap().is_empty());
    assert!(check_saved("policy", "no-threshold.yml")
        .unwrap()
        .is_empty());
    assert!(check_saved("policy", "empty-options.yml")
        .unwrap()
        .is_empty());
    assert!(check_saved("file-disabled", "policy.yml")
        .unwrap()
        .is_empty());
}

#[test]
fn configured_each_calls_message_and_suppression() {
    assert_eq!(
        sorted_messages(&check_saved("each", "each-calls.yml").unwrap()),
        ["test.each size 50000 exceeds threshold 1000"]
    );
    assert_eq!(
        sorted_messages(&check_saved("policy", "no-each.yml").unwrap()),
        [
            "Array.from length 1001 exceeds threshold 1000",
            "Array.from length 2000 exceeds threshold 1000",
            OVER_NEW,
        ]
    );
    assert_eq!(
        sorted_messages(&check_saved("message", "message.yml").unwrap()),
        ["widget fixtures Array.from length 1001 exceeds threshold 1000"]
    );
    let root = saved("fixture", "suppressed");
    let config = load_v2_config(&root, Some(&saved("configs", "policy.yml"))).unwrap();
    let files = discover_files(&root, &[]);
    let sources = super::super::source_store_for_files(&files);
    let deferred =
        super::check_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    let applied =
        super::check_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert_eq!(deferred.len(), 3, "{deferred:?}");
    assert_eq!(
        sorted_messages(&applied),
        ["new Array length 9000 exceeds threshold 1000"]
    );
}

#[test]
fn saved_recall_reports_bare_array_and_literal_siblings() {
    assert_eq!(
        sorted_messages(&check_saved("recall", "it-each.yml").unwrap()),
        [
            "it.each size 10000 exceeds threshold 1000",
            "it.each size 10000 exceeds threshold 1000",
            "it.each size 10000 exceeds threshold 1000",
            "it.each size 10000 exceeds threshold 1000",
            "it.each size 10001 exceeds threshold 1000",
            "it.each size 10001 exceeds threshold 1000",
            OVER_NEW,
        ]
    );
}

#[test]
fn saved_boundary_tables_are_silent_until_each_calls_is_set() {
    assert!(check_saved("boundary", "policy.yml").unwrap().is_empty());
    assert_eq!(
        sorted_messages(&check_saved("boundary", "it-each.yml").unwrap()),
        [
            "it.each size 1001 exceeds threshold 1000",
            "it.each size 20261004 exceeds threshold 1000",
            "it.each size 30001 exceeds threshold 1000",
            "it.each size 3601 exceeds threshold 1000",
        ]
    );
}

#[test]
fn saved_dynamic_length_drops_a_length_a_later_form_can_replace() {
    assert_eq!(
        sorted_messages(&check_saved("dynamic-length", "policy.yml").unwrap()),
        [
            "Array.from length 5000 exceeds threshold 1000",
            "Array.from length 5000 exceeds threshold 1000",
            "Array.from length 5000 exceeds threshold 1000",
        ]
    );
}

#[test]
fn saved_configs_reject_bad_options_before_scanning() {
    let glob = check_saved("policy", "bad-glob.yml")
        .unwrap_err()
        .to_string();
    assert!(glob.contains("scopes.tests"), "{glob}");
    let each = check_saved("policy", "bad-each.yml")
        .unwrap_err()
        .to_string();
    assert!(each.contains("eachCalls"), "{each}");
    let threshold = check_saved("policy", "bad-threshold.yml")
        .unwrap_err()
        .to_string();
    assert!(threshold.contains("threshold"), "{threshold}");
    let mode = check_saved("policy", "bad-each-table-mode.yml")
        .unwrap_err()
        .to_string();
    assert!(mode.contains("options.eachTableMode"), "{mode}");
}
