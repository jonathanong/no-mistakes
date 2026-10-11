use super::*;
use crate::integration_tests::types::{DeadlineUnknownReason, DeadlineValue};

fn projects(file: &str) -> Vec<types::ConfigProject> {
    let root = fixture("declared-deadlines");
    let path = root.join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    parse_vitest_fixture(&source, &path, &root).unwrap()
}

#[test]
fn nested_extends_preserves_setup_ownership_without_claiming_deadline_inheritance() {
    let projects = projects("vitest.setup-deadline-contract.ts");
    for name in ["nested-true", "nested-false", "nested-file"] {
        let project = projects
            .iter()
            .find(|p| p.policy_name.as_deref() == Some(name))
            .unwrap();
        let case = project.declared_deadlines.case.as_ref().unwrap();
        assert_eq!(case.value, DeadlineValue::Known(30000.0));
        assert_eq!(
            project.declared_deadlines.hook.as_ref().unwrap().value,
            DeadlineValue::Known(1.0)
        );
        assert_eq!(case.inherited_through.len(), 1);
        assert!(case.path.ends_with("vitest.setup-deadline-contract.ts"));
        assert!(case.span.is_some());
        if name == "nested-true" {
            assert_eq!(project.vitest_setup.len(), 1);
            assert_eq!(
                project.vitest_setup[0].specifier.as_deref(),
                Some("./root-setup.ts")
            );
        } else if name == "nested-file" {
            assert_eq!(project.vitest_setup.len(), 1);
            assert!(project.vitest_setup[0].config_extends_provenance);
        } else {
            assert!(project.vitest_setup.is_empty());
        }
    }
    let independent = projects
        .iter()
        .find(|p| p.policy_name.as_deref() == Some("outer-false"))
        .unwrap();
    assert!(independent.declared_deadlines.case.is_none());
    let named = projects
        .iter()
        .find(|p| p.policy_name.as_deref() == Some("outer-file"))
        .unwrap();
    assert_eq!(
        named.declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedConfigRoot)
    );
    assert!(named
        .declared_deadlines
        .case
        .as_ref()
        .unwrap()
        .path
        .ends_with("base.config.ts"));
}

#[test]
fn merged_namespace_alias_preserves_discovery_and_setup_but_not_known_deadlines() {
    let projects = projects("vitest.alias-merge-contract.ts");
    assert_eq!(projects.len(), 1);
    let project = &projects[0];
    assert_eq!(project.scope.as_deref(), Some("alias-root"));
    assert_eq!(project.include, vec!["alias-root/owned/*.test.ts"]);
    assert_eq!(project.vitest_setup.len(), 1);
    assert_eq!(
        project.vitest_setup[0].specifier.as_deref(),
        Some("./alias-setup.ts")
    );
    for declaration in [
        &project.declared_deadlines.case,
        &project.declared_deadlines.hook,
    ] {
        assert_eq!(
            declaration.as_ref().unwrap().value,
            DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
        );
        assert!(declaration.as_ref().unwrap().span.is_some());
    }
}

#[test]
fn commonjs_structural_discovery_does_not_claim_public_sdk_deadline_proof() {
    let projects = projects("vitest.commonjs-structure.cjs");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].scope.as_deref(), Some("legacy-root"));
    assert_eq!(projects[0].include, vec!["legacy-root/owned/*.test.ts"]);
    assert_eq!(projects[0].vitest_setup.len(), 1);
    assert_eq!(
        projects[0].vitest_setup[0].specifier.as_deref(),
        Some("./legacy-setup.ts")
    );
    assert_eq!(
        projects[0].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Unknown(DeadlineUnknownReason::UnsupportedConfigCall)
    );
}

#[test]
fn unproved_merge_bindings_retain_both_structural_arguments_without_known_deadlines() {
    for file in ["vitest.bare-merge.ts", "vitest.member-merge.ts"] {
        let projects = projects(file);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].scope.as_deref(), Some("legacy-root"));
        assert_eq!(projects[0].include, vec!["legacy-root/owned/*.test.ts"]);
        assert_eq!(projects[0].vitest_setup.len(), 1);
        assert_eq!(
            projects[0].vitest_setup[0].specifier.as_deref(),
            Some("./legacy-setup.ts")
        );
        for declaration in [
            &projects[0].declared_deadlines.case,
            &projects[0].declared_deadlines.hook,
        ] {
            assert_eq!(
                declaration.as_ref().unwrap().value,
                DeadlineValue::Unknown(DeadlineUnknownReason::UnsupportedConfigCall)
            );
        }
    }
}
