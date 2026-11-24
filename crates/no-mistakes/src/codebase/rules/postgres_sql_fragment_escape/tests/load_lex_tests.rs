use super::super::load_gap::missing_static_load_gap;
use super::super::load_windows::{direct_module_load, matching_parens};
use super::support::{call, named};
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::ts_source::facts::TsFileFacts;

#[test]
fn direct_module_load_skips_regex_literals_with_quotes_and_parens() {
    assert!(direct_module_load(
        "require('./columns', /['\")]+/g)",
        false
    ));
    assert!(!direct_module_load(
        "require('./columns', /['\")]+/g",
        false
    ));
    assert!(!direct_module_load(
        "require('./columns' + 'unterminated)",
        false
    ));
}

#[test]
fn matching_parens_ignores_parens_inside_strings_comments_and_regexes() {
    let source = "f(/'\\)'/g, ')', /* ) */ g(x))";
    let pairs = matching_parens(source);
    let outer = source.find('(').unwrap();
    let inner = source.rfind('(').unwrap();
    assert_eq!(pairs.get(&outer), Some(&(source.len() - 1)));
    assert_eq!(pairs.get(&inner), Some(&(source.len() - 2)));

    assert!(matching_parens("f(/* unterminated").is_empty());
    assert!(matching_parens("x = /unclosed\n(a)").contains_key(&14));
    assert!(matching_parens("x = /trailing\\").is_empty());
    assert!(matching_parens("x = /esc\\\n(a)").contains_key(&10));
}

#[test]
fn saved_slash_forms_pin_visible_call_parens() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/slash-lexing/slash-forms.js"
    ));
    let pairs = matching_parens(source);
    let regex_call = source.find("record(/").expect("regex argument") + "record".len();
    let regex_call_end = source.find("/g);").expect("regex closing slash") + "/g".len();
    assert_eq!(pairs.get(&regex_call), Some(&regex_call_end));
    for (marker, relative_close) in [
        ("require(", None),
        ("afterParen(", Some(1)),
        ("afterBracket(", Some(1)),
        ("afterBrace(", None),
        ("afterIncrement(", None),
    ] {
        let open = source.find(marker).expect(marker) + marker.len() - 1;
        assert_eq!(
            pairs.get(&open).copied(),
            relative_close.map(|relative| open + relative),
            "{marker}"
        );
    }
}

#[test]
fn unclosed_outer_paren_falls_back_to_depth_tracking() {
    let source = "outer(inner(1), require('./columns')";
    let mut ts = TsFileFacts::default();
    ts.value_flow
        .calls
        .push(call(0, source.len() as u32, named("outer"), Vec::new()));
    let inner_start = source.find("inner(").unwrap() as u32;
    let inner_end = inner_start + "inner(1)".len() as u32;
    ts.value_flow
        .calls
        .push(call(inner_start, inner_end, named("inner"), Vec::new()));
    assert_eq!(missing_static_load_gap(&ts, source, "unclosed.ts"), None);
}

#[test]
fn nested_and_chained_calls_detect_innermost_miss_without_quadratic_rescan() {
    let depth = 400usize;
    let mut source = String::new();
    let mut starts = Vec::with_capacity(depth + 1);
    for _ in 0..depth {
        starts.push(source.len());
        source.push_str("wrap(");
    }
    let req_start = source.len();
    source.push_str("require('./columns')");
    let req_end = source.len();
    for _ in 0..depth {
        source.push(')');
    }
    let mut ts = TsFileFacts::default();
    for (idx, start) in starts.into_iter().enumerate() {
        let end = source.len() - idx;
        ts.value_flow
            .calls
            .push(call(start as u32, end as u32, named("wrap"), Vec::new()));
    }
    ts.value_flow.calls.push(call(
        req_start as u32,
        req_end as u32,
        named("require"),
        vec![ValueExpression::Unknown],
    ));
    assert!(
        missing_static_load_gap(&ts, &source, "nested.ts").is_some(),
        "innermost unrecorded require must still be detected"
    );
    ts.namespaces
        .literal_import_spans
        .push((req_start as u32, "./columns".to_string()));
    assert_eq!(missing_static_load_gap(&ts, &source, "nested.ts"), None);
}
