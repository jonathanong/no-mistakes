use super::*;
use crate::integration_tests::types::{DeadlineUnknownReason, DeadlineValue};

fn vitest(file: &str) -> Vec<types::ConfigProject> {
    let root = fixture("declared-deadlines");
    let path = root.join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    parse_vitest_fixture(&source, &path, &root).unwrap()
}

fn project<'a>(projects: &'a [types::ConfigProject], name: &str) -> &'a types::ConfigProject {
    projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some(name))
        .unwrap()
}

#[test]
fn declared_computed_keys_and_opaque_test_replacements_cannot_inherit_known_budgets() {
    let projects = vitest("vitest.uncertainty.ts");
    let computed = &project(&projects, "computed-test").declared_deadlines;
    for slot in [&computed.case, &computed.hook] {
        let declaration = slot.as_ref().unwrap();
        assert_eq!(
            declaration.value,
            DeadlineValue::Unknown(DeadlineUnknownReason::ComputedProperty)
        );
        assert!(declaration.span.is_some());
        assert!(declaration.path.ends_with("vitest.uncertainty.ts"));
    }
    let outer = &project(&projects, "computed-project").declared_deadlines;
    assert_eq!(
        outer.case.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        outer.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnresolvedInheritance)
    );
    let restored = &project(&projects, "computed-restored").declared_deadlines;
    assert_eq!(
        restored.case.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        restored.hook.as_ref().unwrap().value,
        DeadlineValue::Known(30000.0)
    );
    for slot in [
        &project(&projects, "opaque-test").declared_deadlines.case,
        &project(&projects, "opaque-test").declared_deadlines.hook,
    ] {
        assert_eq!(
            slot.as_ref().unwrap().value,
            DeadlineValue::Unknown(DeadlineUnknownReason::OpaqueTestObject)
        );
    }
}

#[test]
fn declared_hook_test_and_extends_getters_do_not_execute_or_restore_parent_defaults() {
    let projects = vitest("vitest.accessors.ts");
    let hook = &project(&projects, "hook-getter").declared_deadlines;
    assert_eq!(
        hook.case.as_ref().unwrap().value,
        DeadlineValue::Known(30000.0)
    );
    assert_eq!(
        hook.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::Accessor)
    );
    let test = &project(&projects, "test-getter").declared_deadlines;
    for slot in [&test.case, &test.hook] {
        assert_eq!(
            slot.as_ref().unwrap().value,
            DeadlineValue::Unknown(DeadlineUnknownReason::Accessor)
        );
    }
    let extends = &project(&projects, "extends-getter").declared_deadlines;
    assert_eq!(
        extends.case.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        extends.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnresolvedInheritance)
    );
}

#[test]
fn declared_unary_non_numeric_expressions_remain_unknown() {
    let projects = vitest("vitest.uncertainty.ts");
    let deadlines = &project(&projects, "unary-operator").declared_deadlines;
    for slot in [&deadlines.case, &deadlines.hook] {
        assert_eq!(
            slot.as_ref().unwrap().value,
            DeadlineValue::Unknown(DeadlineUnknownReason::Expression)
        );
    }
}

#[test]
fn declared_playwright_computed_keys_obscure_only_case_until_explicit_override() {
    let root = fixture("declared-deadlines");
    let path = root.join("playwright.computed.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_playwright_fixture(&source, &path, &root)
        .unwrap()
        .into_projects(&root, "playwright.computed.ts");
    let computed = &project(&projects, "computed").declared_deadlines;
    assert_eq!(
        computed.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::ComputedProperty)
    );
    assert!(computed.hook.is_none() && computed.fixture.is_none());
    assert_eq!(
        project(&projects, "restored")
            .declared_deadlines
            .case
            .as_ref()
            .unwrap()
            .value,
        DeadlineValue::Known(1.0)
    );
}
