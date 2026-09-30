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

/// The graph cannot resolve a reference to a namespace member, such as
/// `new Built.Qualified()` or a bare `new Local()` inside its namespace, so it
/// cannot tell a namespaced class is built. `declare` classes and classes in an
/// ambient module block describe code outside the analyzed source. Each of
/// these is an exported error class that nothing else constructs, so the rule
/// stays silent on all of them.
#[test]
fn namespaced_and_ambient_classes_are_never_reported() {
    let found = findings(".no-mistakes.yml").unwrap();
    let targets: Vec<_> = found
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();
    for silent in [
        "DeadNamespacedError",
        "DeadDottedError",
        "TopicError",
        "DeepError",
        "CollideBase",
        "Qualified",
        "Local",
        "Hidden",
        "DeclaredError",
        "ModuleBlockError",
    ] {
        assert!(!targets.contains(&silent), "{silent} was reported");
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

/// Declaration files describe code outside the analyzed source, so nothing
/// constructing their classes is expected to be visible. Each extension has its
/// own fixture: a `.d.ts`, `.d.mts`, and `.d.cts` class that nothing builds.
#[test]
fn classes_declared_in_declaration_files_are_never_reported() {
    let found = reported(&findings(".no-mistakes.yml").unwrap()).join("\n");
    assert!(!found.contains("src/ambient."), "{found}");
    for ambient in [
        "AmbientClientError",
        "AmbientModuleError",
        "AmbientCommonError",
    ] {
        assert!(!found.contains(ambient), "{ambient} was reported");
    }
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
        "CollideChild",
        "CollideGrand",
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
fn graph_plan_requests_calls_and_extends_when_configured() {
    let root = fixture();
    let configured =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let plan = graph_plan(&configured).expect("configured rule needs a graph");
    assert!(plan.calls && plan.extends);
    assert!(graph_plan(&NoMistakesConfig::default()).is_none());
}

/// A file the graph could not parse might hold the only construction of a
/// class, so the rule errors instead of reporting from an incomplete view.
#[test]
fn a_broken_source_file_is_a_rule_error_naming_the_file() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/parse-failure/fixture"),
    );
    let config = root.join("configs/one-broken-source.yml");
    let error = run_check(&root, Some(&config), None)
        .unwrap_err()
        .to_string();
    assert!(
        error.starts_with(
            "unconstructed-error-class: cannot prove error classes unconstructed: \
             `src/broken.ts` failed to parse: "
        ),
        "{error}"
    );
}
