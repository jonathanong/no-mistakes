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
            // `bind`, `call` and `apply` hand a class on; a static guard does not.
            "src/namespace-bound.ts:25 Guarded.GuardedDead",
            // A constant or a parameter that shares a name is not the namespace.
            "src/namespace-collision.ts:7 Collide.Inner.CollideDead",
            "src/namespace-collision.ts:16 Hide.HideDead",
            // Namespace members: reported by qualified name, like any class.
            "src/namespace-construct.ts:23 Sub.Child",
            "src/namespace-consumer-lib.ts:30 Standard.StandardDead",
            "src/namespace-lib.ts:9 Lib.DeadLibError",
            "src/namespace-lib.ts:21 Lib.TestOnlyBuilt",
            "src/namespace-lib.ts:31 Lib.Deep.DeepDead",
            "src/namespace-lib.ts:40 Renamed.RenamedDead",
            "src/namespace-merged.ts:10 Merged.Second",
            "src/namespace-merged.ts:23 Mix.Inner.NestedPart",
            // A barrel read whole exposes only what it re-exports.
            "src/namespace-selective-target.ts:4 Kept.KeptDead",
            // A parameter that shadows the namespace or a class is not a use.
            "src/namespace-shadowed.ts:4 Shadowed.ShadowDead",
            "src/namespace-shadowed.ts:6 Shadowed.InnerShadow",
            // A sourced clause exports the target's namespace, not a local one.
            "src/namespace-sourced-target.ts:5 Unseen.RemoteDead",
            // `typeof`, `implements` and an interface base are erased types.
            "src/namespace-type-names.ts:4 Queried.QueriedDead",
            "src/namespace-type-names.ts:10 Marked.MarkedDead",
            "src/namespace-type-names.ts:16 Extended.ExtendedDead",
            // `import type X = require()` is erased, so it uses nothing.
            "src/namespace-type-only-target.ts:2 TypeOnly.TypeOnlyError",
            "src/namespaced.ts:14 Errors.DeadNamespacedError",
            "src/namespaced.ts:30 Errors.Child",
            "src/namespaced.ts:34 Errors.Inner.DeepDeadError",
            "src/namespaced.ts:53 TopLevelChild",
            "src/namespaced.ts:57 Dotted.Path.DeadDottedError",
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
        "NestedFactoryError",
        "NamedFactoryError",
    ] {
        assert!(!found.contains(constructed), "{constructed} was reported");
    }
}

fn targets(findings: &[RuleFinding]) -> Vec<&str> {
    findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect()
}

/// A namespace member is built through `new Errors.Built()`, a bare `new
/// Local()` in the namespace body, a nested or dotted path, and an import of the
/// namespace by name, rename, namespace import, or barrel. Each of these classes
/// is an exported error class that no other code builds, so the rule would
/// report it if the graph did not resolve the reference.
#[test]
fn namespace_members_built_through_a_resolved_reference_are_not_reported() {
    let found = findings(".no-mistakes.yml").unwrap();
    let targets = targets(&found);
    for silent in [
        // Same file: qualified, bare, nested, dotted, and `new this()`.
        "Errors.Built",
        "Errors.Local",
        "Errors.TopicError",
        "Errors.Inner.DeepBuilt",
        "Errors.Inner.DeepError",
        "Dotted.Path.BuiltDotted",
        "Merged.First",
        "Mix.Inner.DottedPart",
        // Another file, by every way to reach the namespace.
        "Lib.Used",
        "Lib.Deep.DeepUsed",
        "Lib.ViaImportRename",
        "Lib.ViaStar",
        "Lib.ViaBarrel",
        "Renamed.ViaExportRename",
        // A default import of a namespace exported as `default`.
        "Standard.Built",
        // Through a sourced clause: `Public` is the target's namespace.
        "Unseen.RemoteLive",
    ] {
        assert!(!targets.contains(&silent), "{silent} was reported");
    }
}

/// A namespace subclass credits its base the way a top-level subclass does,
/// whether the `extends` is written in the namespace body (`extends Base`) or
/// outside it (`extends Errors.Base`, `extends Lib.Base`). Nothing constructs
/// the bases, so each is silent only because that `extends` credits it.
#[test]
fn a_subclass_of_a_namespace_member_credits_its_base() {
    let found = findings(".no-mistakes.yml").unwrap();
    let targets = targets(&found);
    for credited in ["Errors.Base", "Lib.Base", "NsBase"] {
        assert!(!targets.contains(&credited), "{credited} was reported");
    }
    // A top-level class that extends `Errors.Base` is an error class, and a
    // dead one, even though its base is in a namespace.
    assert!(targets.contains(&"TopLevelChild"));
    assert!(targets.contains(&"Sub.Child"));
}

/// A namespace is reported on only while every use of it is a static member
/// access the graph resolves. Each namespace below has a dead class that would
/// be flagged but for one use that could reach the class: an alias, a
/// destructuring, an argument, a computed access, a default export, a merge with
/// a class, a module namespace used as a value, a dynamic import or `require`,
/// and a construction of a member the graph cannot find.
#[test]
fn a_namespace_that_escapes_is_never_reported() {
    let found = findings(".no-mistakes.yml").unwrap();
    let targets = targets(&found);
    for silent in [
        "Aliased.AliasedDead",
        "Picked.PickedDead",
        "Destructured.DestructuredDead",
        "Passed.PassedDead",
        "Argument.ArgumentDead",
        "Computed.ComputedDead",
        "Mixed.MixedDead",
        "Defaulted.DefaultedDead",
        "Whole.WholeDead",
        "Dynamic.DynamicDead",
        "Required.RequiredDead",
        "Backstop.BackstopDead",
        "Gap.GapDead",
        // An import and a namespace of one name.
        "Imported.ImportMergedError",
        // A member handed on by `bind`, `call` or `apply`, in the file or
        // through an import.
        "Bound.BoundDead",
        "Called.CalledDead",
        "Applied.AppliedDead",
        "Handed.HandedDead",
        // A bare name in the body that denotes the nested namespace.
        "Reach.Inner.ReachDead",
        // A barrel that re-exports the namespace, read whole.
        "Exposed.ExposedDead",
        // Another file imports the namespace, and uses it as a value.
        "ViaAlias.ViaAliasDead",
        "ViaArgument.ViaArgumentDead",
        "ViaComputed.ViaComputedDead",
        "ViaMember.ViaMemberDead",
    ] {
        assert!(!targets.contains(&silent), "{silent} was reported");
    }
}

/// `declare` classes, classes in a `declare namespace`, an ambient module
/// block, or `declare global`, and a script file's global namespace describe
/// code outside the analyzed source or are not exports of the file. Each is an
/// error class that nothing builds, and none is reported. A member its
/// namespace does not export, a class declared in a function or as an
/// expression, and a member of an unexported namespace are not exports either.
#[test]
fn ambient_and_unexported_namespace_classes_are_never_reported() {
    let found = reported(&findings(".no-mistakes.yml").unwrap()).join("\n");
    for silent in [
        "DeclaredError",
        "ModuleBlockError",
        "AmbientNamespaceError",
        "GlobalAugmentationError",
        "ScriptDead",
        "LegacyDead",
        // In a namespace that only a sourced clause of the same name exports.
        "LocalUnseen",
        "Hidden",
        "InternalError",
        "LocalError",
        "ExpressionMemberError",
        "CollideBase",
    ] {
        assert!(!found.contains(silent), "{silent} was reported: {found}");
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
