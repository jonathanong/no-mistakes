use crate::playwright::selectors::{extract_app_selectors, AppSelector};
use crate::playwright::test_support::fixture_path;
use std::collections::BTreeMap;
use std::path::Path;

fn attrs() -> Vec<String> {
    vec!["data-testid".to_string(), "data-pw".to_string()]
}

#[test]
fn extracts_static_identifier_default_jsx_selectors() {
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        r#"
        export function Link({ 'data-pw': dataPw = 'rss-feed-link' }) {
            return <a data-pw={dataPw}>RSS</a>;
        }
        export function Button({ passThrough }) {
            return (
                <>
                    <button data-pw={passThrough}>Save</button>
                    <button data-pw={1 + 1}>Count</button>
                </>
            );
        }
        export function DynamicLink({ dataPw }) {
            return <a data-pw={dataPw}>Dynamic</a>;
        }
        export function FullyDynamicTemplate({ id }) {
            return <a data-pw={`${id}`}>Dynamic template</a>;
        }
        export const ArrowLink = ({ dataPw = 'arrow-link' }) => {
            return <a data-pw={dataPw}>Arrow</a>;
        };
        export function DirectDefault(dataPw = 'direct-link') {
            return <a data-pw={dataPw}>Direct</a>;
        }
        export function ArrayDefault([dataPw = 'array-link']) {
            return <a data-pw={dataPw}>Array</a>;
        }
        export function NonStringDefault({ value = makeId() }) {
            return <a data-pw={value}>Computed</a>;
        }
        export function NestedShadow({ dataPw = 'outer-link' }) {
            function Inner({ dataPw }) { return <a data-pw={dataPw}>Inner</a>; }
            return <Inner />;
        }
        export function Reassigned({ reassigned = 'assigned-link' }) {
            reassigned = makeId();
            return <a data-pw={reassigned}>Assigned</a>;
        }
        export function CompoundReassigned({ compound = 'compound-link' }) {
            compound += '-dynamic';
            return <a data-pw={compound}>Compound</a>;
        }
        export function DestructuredShadow({ shadowed = 'shadowed-link' }, props) {
            const { shadowed } = props;
            return <a data-pw={shadowed}>Shadowed</a>;
        }
        export function CommentAndStringText({ dataPw = 'comment-safe-link' }) {
            // dataPw = makeId();
            const message = "dataPw = makeId();";
            return <a data-pw={dataPw}>Comment safe</a>;
        }
        export function TemplateExpressionMutation({ mutated = 'template-mutation-link' }) {
            const label = `${mutated = makeId()}`;
            return <a data-pw={mutated}>Template mutation</a>;
        }
        export function EarlierHelperParam({ dataPw = 'helper-param-link' }) {
            function helper(dataPw) { return dataPw; }
            const local = (dataPw) => dataPw;
            return <a data-pw={dataPw}>{helper(local('x'))}</a>;
        }
        export function WithHelper({ dataPw = 'helper-link' }) {
            const isReady = () => dataPw === 'helper-link';
            return isReady() ? <a data-pw={dataPw}>Ready</a> : null;
        }
        export function ShortName({ id = 'short-link' }) {
            const userId = makeId();
            return <a data-pw={id}>Short</a>;
        }
        "#,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();

    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(
        values,
        vec![
            "array-link",
            "arrow-link",
            "comment-safe-link",
            "direct-link",
            "helper-link",
            "helper-param-link",
            "rss-feed-link",
            "short-link",
            "{${id}}",
            "{1 + 1}",
            "{compound}",
            "{dataPw}",
            "{mutated}",
            "{passThrough}",
            "{reassigned}",
            "{shadowed}",
            "{value}",
        ]
    );
}

#[test]
fn resolves_ternary_initializer() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "dynamic-ternary.tsx",
    ]);
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        &source,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(values, vec!["inline-a", "inline-b", "option-a", "option-b"]);
}

#[test]
fn resolves_if_else_assignment() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "dynamic-conditional.tsx",
    ]);
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        &source,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(values, vec!["branch-a", "branch-b"]);
}

#[test]
fn resolves_object_map() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "dynamic-object-map.tsx",
    ]);
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        &source,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(values, vec!["val-a", "val-b", "val-c"]);
}

#[test]
fn resolves_function_return() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "dynamic-function-return.tsx",
    ]);
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        &source,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(values, vec!["fn-a", "fn-b"]);
}

#[test]
fn resolves_cross_file_imports() {
    let page_path = fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]);
    let source = std::fs::read_to_string(&page_path).unwrap();
    let selectors = extract_app_selectors(&page_path, &source, &attrs(), &BTreeMap::new()).unwrap();
    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(
        values,
        vec![
            "imported-const",
            "imported-fn-val",
            "imported-obj-a",
            "imported-obj-b"
        ]
    );
}

// ── jsx_resolve: None value (boolean attribute) ──────────────────────────────

#[test]
fn boolean_jsx_attribute_no_value_produces_no_selector() {
    // `data-pw` with no value → JSXAttributeValue is None → app_selector_values returns empty
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        r#"
        export function BoolAttr() {
          return <button data-pw />;
        }
        "#,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    assert!(selectors.is_empty());
}

// ── jsx_resolve: inline ConditionalExpression with string leaves ──────────────

#[test]
fn inline_ternary_with_string_leaves_resolves_both_branches() {
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        r#"
        export function InlineAll({ cond }) {
          return <button data-pw={cond ? 'direct-a' : 'direct-b'} />;
        }
        "#,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(values, vec!["direct-a", "direct-b"]);
}

// ── jsx_resolve: inline ConditionalExpression with no string leaves ───────────

#[test]
fn inline_ternary_with_no_string_leaves_produces_unsupported() {
    // Both branches are identifiers → no string leaves → Unsupported
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        r#"
        export function NonStringTernary({ cond, a, b }) {
          return <button data-pw={cond ? a : b} />;
        }
        "#,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0], "{cond ? a : b}");
}

// ── jsx_resolve: inline LogicalExpression with string leaves ─────────────────

#[test]
fn inline_logical_with_string_leaves_resolves_both_sides() {
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        r#"
        export function InlineLogical({ cond }) {
          return <button data-pw={'logical-a' || 'logical-b'} />;
        }
        "#,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let mut values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    values.sort();
    values.dedup();
    assert_eq!(values, vec!["logical-a", "logical-b"]);
}

// ── jsx_resolve: inline LogicalExpression with no string leaves ───────────────

#[test]
fn inline_logical_with_no_string_leaves_produces_unsupported() {
    // Both sides are identifiers → no string leaves → Unsupported
    let selectors = extract_app_selectors(
        Path::new("app/page.tsx"),
        r#"
        export function NoStringLogical({ a, b }) {
          return <button data-pw={a || b} />;
        }
        "#,
        &attrs(),
        &BTreeMap::new(),
    )
    .unwrap();
    let values: Vec<String> = selectors.iter().map(AppSelector::display_value).collect();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0], "{a || b}");
}
