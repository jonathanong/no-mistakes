use super::*;
use crate::codebase::rules::{run_check, RuleFinding};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/fixture"),
    )
}

fn findings(config: &str) -> anyhow::Result<Vec<RuleFinding>> {
    let root = fixture();
    run_check(&root, Some(&root.join(config)), None)
}

/// `file:line name` for each finding, in report order.
fn reported(findings: &[RuleFinding]) -> Vec<String> {
    findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.file,
                finding.line,
                finding.target.as_deref().unwrap_or("?")
            )
        })
        .collect()
}

#[test]
fn default_config_reports_exactly_the_dead_error_classes() {
    let found = findings(".no-mistakes.yml").unwrap();
    assert_eq!(
        reported(&found),
        [
            // Workspace package: only the class nothing constructs.
            "packages/lib/index.ts:5 WorkspaceUnusedError",
            "src/anonymous.ts:2 default",
            // Only a test, or only a `__tests__` file, builds these.
            "src/errors.ts:2 UnusedError",
            "src/errors.ts:8 TestOnlyError",
            // Built-in subclasses and `globalThis.Error`.
            "src/errors.ts:11 UnusedRangeError",
            "src/errors.ts:14 UnusedGlobalError",
            // Export clauses, renamed exports, and default exports.
            "src/errors.ts:34 ClauseUnusedError",
            "src/errors.ts:38 RenamedExportError",
            "src/errors.ts:42 DefaultUnusedError",
            // `instanceof` and a type guard are not construction.
            "src/guard.ts:3 InstanceofOnlyError",
            // The leaf and the orphan base; the middle of the chain is satisfied.
            "src/hierarchy.ts:11 Grandchild",
            "src/hierarchy.ts:14 OrphanBase",
            "src/reexported.ts:14 BarrelUnusedError",
            "src/workspace-use.ts:6 LocalFromLib",
        ]
    );
    assert!(found.iter().all(|finding| finding.rule == RULE_ID));
}

#[test]
fn constructions_through_aliases_barrels_namespaces_and_workspaces_count() {
    let found = reported(&findings(".no-mistakes.yml").unwrap()).join("\n");
    for constructed in [
        "ConstructedError",
        "SameFileError",
        "ChildError",
        "AliasedError",
        "BarrelError",
        "StarError",
        "NamespaceError",
        "WorkspaceError",
        "ThisFactoryError",
        "NamedFactoryError",
    ] {
        assert!(!found.contains(constructed), "{constructed} was reported");
    }
}

#[test]
fn subclassing_satisfies_a_base_even_when_the_subclass_is_dead() {
    let found = reported(&findings(".no-mistakes.yml").unwrap()).join("\n");
    for satisfied in ["AppError", "MiddleError", "LibBaseError"] {
        assert!(!found.contains(satisfied), "{satisfied} was reported");
    }
    assert!(found.contains("Grandchild"));
}

#[test]
fn unrelated_and_opaque_classes_are_ignored() {
    let found = reported(&findings(".no-mistakes.yml").unwrap()).join("\n");
    for ignored in [
        "PrivateUnusedError",
        "NotAnError",
        "ExtendsNotAnError",
        "ExternalBaseError",
        "MixinError",
        "ExpressionError",
        "CycleA",
        "CycleB",
        "SuppressedError",
    ] {
        assert!(!found.contains(ignored), "{ignored} was reported");
    }
}

#[test]
fn shared_test_helpers_count_as_source_until_test_files_says_otherwise() {
    let default = reported(&findings(".no-mistakes.yml").unwrap());
    assert!(!default.iter().any(|line| line.contains("HelperOnlyError")));

    let configured = reported(&findings("configs/test-files.yml").unwrap());
    assert!(configured.contains(&"src/helper-only.ts:3 HelperOnlyError".to_string()));
    assert_eq!(configured.len(), default.len() + 1);
}

#[test]
fn include_selects_declarations_but_not_constructions() {
    let found = reported(&findings("configs/include.yml").unwrap());
    // `WorkspaceError` is declared in the included package and constructed
    // from `src/`, which the include does not cover.
    assert_eq!(found, ["packages/lib/index.ts:5 WorkspaceUnusedError"]);
}

#[test]
fn default_messages_name_the_class_or_the_default_export() {
    let found = findings(".no-mistakes.yml").unwrap();
    let message = |target: &str| {
        found
            .iter()
            .find(|finding| finding.target.as_deref() == Some(target))
            .map(|finding| finding.message.as_str())
            .unwrap()
    };
    assert_eq!(
        message("UnusedError"),
        "exported error class `UnusedError` is never constructed or subclassed in non-test source"
    );
    assert_eq!(
        message("default"),
        "default-exported error class is never constructed or subclassed in non-test source"
    );
}

#[test]
fn graph_plan_requests_only_the_class_hierarchy_when_configured() {
    let root = fixture();
    let configured =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let plan = graph_plan(&configured).expect("configured rule needs a graph");
    assert!(plan.class_hierarchy && !plan.calls);
    assert!(graph_plan(&NoMistakesConfig::default()).is_none());
}
