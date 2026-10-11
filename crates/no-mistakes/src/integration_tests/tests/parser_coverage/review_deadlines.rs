use super::*;
use crate::integration_tests::types::{DeadlineUnknownReason, DeadlineValue};

fn projects(file: &str) -> Vec<types::ConfigProject> {
    let root = fixture("declared-deadlines");
    let path = root.join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    parse_playwright_fixture(&source, &path, &root)
        .unwrap()
        .into_projects(&root, file)
}

#[test]
fn review_deadlines_playwright_mutable_root_and_wrapped_alias_are_unknown() {
    for file in [
        "playwright.mutable.ts",
        "playwright.wrapped-alias.ts",
        "playwright.project-spread.ts",
    ] {
        let projects = projects(file);
        assert!(!projects.is_empty());
        for project in projects {
            let declaration = project.declared_deadlines.case.unwrap();
            assert_eq!(
                declaration.value,
                DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
            );
            assert!(declaration.span.is_some());
            assert!(declaration.path.ends_with(file));
            assert!(project.declared_deadlines.hook.is_none());
            assert!(project.declared_deadlines.fixture.is_none());
        }
    }
}

#[test]
fn review_deadlines_playwright_alias_and_helper_projections_preserve_direct_overrides() {
    let projected = projects("playwright.projections.ts");
    for name in [
        "mutable", "mutated", "helper", "implicit", "direct", "restored",
    ] {
        let project = projected
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap();
        let expected = if matches!(name, "direct" | "restored") {
            DeadlineValue::Known(1.0)
        } else {
            DeadlineValue::Unknown(DeadlineUnknownReason::UnprovedBinding)
        };
        assert_eq!(
            project.declared_deadlines.case.as_ref().unwrap().value,
            expected
        );
    }
    let direct = projects("roots/static/playwright.config.ts");
    assert_eq!(
        direct[0].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Known(30000.0)
    );
}
