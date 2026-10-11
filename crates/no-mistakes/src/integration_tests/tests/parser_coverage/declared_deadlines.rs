use super::*;
use crate::integration_tests::types::{DeadlineUnknownReason, DeadlineValue, DeclaredDeadlines};

fn vitest(file: &str) -> Vec<types::ConfigProject> {
    let root = fixture("declared-deadlines");
    let path = root.join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    parse_vitest_fixture(&source, &path, &root).unwrap()
}

fn slots<'a>(projects: &'a [types::ConfigProject], name: &str) -> &'a DeclaredDeadlines {
    &projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some(name))
        .unwrap()
        .declared_deadlines
}

#[test]
fn declared_vitest_budgets_retain_invalid_numbers_and_inheritance() {
    let projects = vitest("vitest.config.ts");
    let inherited = slots(&projects, "inherited");
    let case = inherited.case.as_ref().unwrap();
    assert_eq!(
        case.value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
    assert_eq!(
        inherited.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
    assert!(case.path.ends_with("budgets.ts"));
    assert_eq!(case.inherited_through.len(), 1);
    assert_eq!(
        case.inherited_through[0].project.as_deref(),
        Some("inherited")
    );
    assert!(case.span.is_some());
    let invalid = slots(&projects, "invalid");
    assert_eq!(
        invalid.case.as_ref().unwrap().value,
        DeadlineValue::Known(0.0)
    );
    assert_eq!(
        invalid.hook.as_ref().unwrap().value,
        DeadlineValue::Known(30001.0)
    );
    assert!(invalid.case.as_ref().unwrap().inherited_through.is_empty());
    assert_eq!(
        slots(&projects, "negative").case.as_ref().unwrap().value,
        DeadlineValue::Known(-1.0)
    );
    assert_eq!(slots(&projects, "absent"), &DeclaredDeadlines::default());
    assert!(projects
        .iter()
        .all(|project| project.declared_deadlines.fixture.is_none()));
}

#[test]
fn declared_vitest_spread_order_and_unknown_values_are_not_absence() {
    let projects = vitest("vitest.config.ts");
    let opaque = slots(&projects, "opaque-last");
    assert_eq!(
        opaque.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::OpaqueSpread)
    );
    assert_eq!(
        opaque.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::OpaqueSpread)
    );
    let restored = slots(&projects, "restored");
    assert_eq!(
        restored.case.as_ref().unwrap().value,
        DeadlineValue::Known(30000.0)
    );
    assert_eq!(
        restored.hook.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    let unknown = slots(&projects, "unknown-expression");
    assert_eq!(
        unknown.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::Expression)
    );
    assert_eq!(
        unknown.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::NonFinite)
    );
}

#[test]
fn declared_nested_test_spread_replaces_the_previous_object() {
    let projects = vitest("vitest.nested.ts");
    assert_eq!(
        projects[0].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
    assert_eq!(
        projects[0].declared_deadlines.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
}

#[test]
fn declared_playwright_projects_keep_case_evidence_without_inventing_other_slots() {
    let root = fixture("declared-deadlines");
    let path = root.join("playwright.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_playwright_fixture(&source, &path, &root)
        .unwrap()
        .into_projects(&root, "playwright.config.ts");
    for (name, value) in [
        ("one", 1.0),
        ("zero", 0.0),
        ("high", 30001.0),
        ("restored", 30000.0),
    ] {
        assert_eq!(
            slots(&projects, name).case.as_ref().unwrap().value,
            DeadlineValue::Known(value)
        );
    }
    let inherited = slots(&projects, "inherited").case.as_ref().unwrap();
    assert_eq!(inherited.value, DeadlineValue::Known(30000.0));
    assert_eq!(inherited.inherited_through.len(), 1);
    assert_eq!(
        slots(&projects, "opaque-last").case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::OpaqueSpread)
    );
    assert!(projects
        .iter()
        .all(|project| project.declared_deadlines.hook.is_none()
            && project.declared_deadlines.fixture.is_none()));
}

#[test]
fn declared_config_extends_preserves_origin_and_missing_base_uncertainty() {
    let projects = vitest("vitest.extends.ts");
    let base = slots(&projects, "base");
    let case = base.case.as_ref().unwrap();
    assert_eq!(
        case.value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedConfigRoot)
    );
    assert!(case.path.ends_with("base.config.ts"));
    assert_eq!(case.inherited_through.len(), 1);
    assert_eq!(base.hook.as_ref().unwrap().value, DeadlineValue::Known(0.0));
    let missing = slots(&projects, "missing");
    assert_eq!(
        missing.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnresolvedExtends)
    );
    assert_eq!(
        missing.hook.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
}

#[test]
fn declared_literal_keys_and_getters_keep_slot_specific_evidence() {
    let projects = vitest("vitest.edges.ts");
    assert_eq!(
        slots(&projects, "literal").case.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        slots(&projects, "literal").hook.as_ref().unwrap().value,
        DeadlineValue::Known(30000.0)
    );
    assert_eq!(
        slots(&projects, "invalid-literal")
            .case
            .as_ref()
            .unwrap()
            .value,
        DeadlineValue::Known(0.0)
    );
    assert_eq!(
        slots(&projects, "invalid-literal")
            .hook
            .as_ref()
            .unwrap()
            .value,
        DeadlineValue::Known(30001.0)
    );
    assert_eq!(
        slots(&projects, "getter").case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::Accessor)
    );
    assert_eq!(
        slots(&projects, "getter").hook.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        slots(&projects, "unrelated-getter")
            .case
            .as_ref()
            .unwrap()
            .value,
        DeadlineValue::Known(30000.0)
    );
}

#[test]
fn declared_unknown_inheritance_cannot_become_default_parent_inheritance() {
    let projects = vitest("vitest.edges.ts");
    for name in ["dynamic-extends", "opaque-extends"] {
        assert_eq!(
            slots(&projects, name).case.as_ref().unwrap().value,
            DeadlineValue::Known(1.0)
        );
        assert_eq!(
            slots(&projects, name).hook.as_ref().unwrap().value,
            DeadlineValue::Unknown(DeadlineUnknownReason::UnresolvedInheritance)
        );
    }
    assert!(slots(&projects, "false-restored").hook.is_none());
    assert_eq!(
        slots(&projects, "ignored-test-extends")
            .case
            .as_ref()
            .unwrap()
            .value,
        DeadlineValue::Known(30000.0)
    );
}

#[test]
fn declared_root_opaque_and_test_replacement_keep_uncertainty() {
    let projects = vitest("vitest.root-opaque.ts");
    assert_eq!(
        slots(&projects, "local").case.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        slots(&projects, "local").hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::OpaqueSpread)
    );
    let replaced = vitest("vitest.replacement.ts");
    assert_eq!(
        replaced[0].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::Expression)
    );
    assert!(replaced[0].declared_deadlines.hook.is_none());
}

#[test]
fn declared_config_value_aliases_remain_unknown_without_binding_proof() {
    let merged = vitest("vitest.deep-merge.ts");
    assert_eq!(
        merged[0].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
    assert_eq!(
        merged[0].declared_deadlines.hook.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
    let unsupported = vitest("vitest.raw-call.ts");
    assert_eq!(
        unsupported[0]
            .declared_deadlines
            .case
            .as_ref()
            .unwrap()
            .value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
}

#[test]
fn declared_playwright_literal_and_accessor_keys_preserve_other_evidence() {
    let root = fixture("declared-deadlines");
    let path = root.join("playwright.edges.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_playwright_fixture(&source, &path, &root)
        .unwrap()
        .into_projects(&root, "playwright.edges.ts");
    assert_eq!(
        slots(&projects, "literal").case.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        slots(&projects, "getter").case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::Accessor)
    );
    assert_eq!(
        slots(&projects, "unrelated").case.as_ref().unwrap().value,
        DeadlineValue::Known(30000.0)
    );
}

#[test]
fn declared_public_merge_uses_two_configs_and_boolean_root_flag_only() {
    for file in ["vitest.merge-direct.ts", "vitest.merge-flag.ts"] {
        let projects = vitest(file);
        assert_eq!(
            projects[0].declared_deadlines.case.as_ref().unwrap().value,
            DeadlineValue::Known(30001.0)
        );
        assert_eq!(
            projects[0].declared_deadlines.hook.as_ref().unwrap().value,
            DeadlineValue::Known(1.0)
        );
    }
    for file in [
        "vitest.merge-third-object.ts",
        "vitest.merge-extra.ts",
        "vitest.merge-opaque-flag.ts",
    ] {
        let projects = vitest(file);
        assert_eq!(
            projects[0].declared_deadlines.case.as_ref().unwrap().value,
            DeadlineValue::Unknown(DeadlineUnknownReason::UnsupportedConfigCall)
        );
    }
}

#[test]
fn declared_mutable_and_const_object_aliases_require_canonical_binding_proof() {
    for file in [
        "vitest.mutable-alias.ts",
        "vitest.const-mutated.ts",
        "vitest.deep-merge.ts",
    ] {
        let projects = vitest(file);
        assert_eq!(
            projects[0].declared_deadlines.case.as_ref().unwrap().value,
            DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
        );
    }
}

#[test]
fn declared_shadowed_helper_and_mutable_commonjs_call_cannot_use_public_import_identity() {
    let shadowed = vitest("vitest.shadowed-helper.ts");
    assert_eq!(
        shadowed[0].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
    );
    assert_eq!(
        shadowed[0].declared_deadlines.hook.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    let mutable = vitest("vitest.commonjs-mutable.ts");
    assert_eq!(
        mutable[0].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnsupportedConfigCall)
    );
}
