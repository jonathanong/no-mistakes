use super::*;
use crate::codebase::dependencies::extract::{CallTargetIdentity, CallableId};

fn build(calls: bool) -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("class-bases"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn assert_module_export(
    actual: &ResolvedCallTarget,
    specifier: &str,
    export_path: &str,
    target: Option<(&Path, &str)>,
) {
    let ResolvedCallTarget::ModuleExport {
        specifier: actual_specifier,
        export_path: actual_export,
        repository_target,
        ..
    } = actual
    else {
        panic!("expected a module export, got {actual:?}");
    };
    assert_eq!(
        (actual_specifier.as_str(), actual_export.as_str()),
        (specifier, export_path)
    );
    assert_eq!(
        repository_target
            .as_ref()
            .map(|(file, scope)| (file.as_path(), scope.as_str())),
        target,
        "{actual:?}"
    );
}

#[test]
fn class_bases_record_owner_line_export_state_and_resolved_base() {
    let (root, graph) = build(true);
    let classes = root.join("src/classes.ts");
    let base = root.join("src/base.ts");
    let lib = root.join("packages/lib/index.ts");
    let summary: Vec<_> = graph
        .resolved_class_bases()
        .iter()
        .map(|class| {
            (
                class.file.strip_prefix(&root).unwrap().to_str().unwrap(),
                class.class_scope.as_str(),
                class.exported,
                class.line,
                class.source_base.as_str(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("packages/lib/index.ts", "LibError", true, 1, "Error"),
            ("src/classes.ts", "Local", true, 6, "Base"),
            ("src/classes.ts", "FromNamespace", true, 7, "base.Base"),
            ("src/classes.ts", "FromWorkspace", true, 8, "LibError"),
            ("src/classes.ts", "SameFile", true, 9, "Local"),
            ("src/classes.ts", "Builtin", true, 10, "TypeError"),
            ("src/classes.ts", "FromPackage", true, 11, "Missing"),
            ("src/classes.ts", "Private", true, 15, "Error"),
            ("src/classes.ts", "default", true, 18, "Error"),
        ]
    );

    let target = |scope: &str| {
        &graph
            .resolved_class_bases()
            .iter()
            .find(|class| class.class_scope == scope)
            .unwrap()
            .base
    };
    assert_module_export(target("Local"), "./base.js", "Base", Some((&base, "Base")));
    assert_module_export(
        target("FromNamespace"),
        "./base.js",
        "Base",
        Some((&base, "Base")),
    );
    assert_module_export(
        target("FromWorkspace"),
        "@fixture/lib",
        "LibError",
        Some((&lib, "LibError")),
    );
    assert_module_export(target("FromPackage"), "external-package", "Missing", None);
    assert_eq!(
        target("SameFile"),
        &ResolvedCallTarget::RepositoryFunction {
            file: classes,
            scope: "Local".to_string()
        }
    );
    assert_eq!(
        target("Builtin"),
        &ResolvedCallTarget::Global {
            name: "TypeError".to_string()
        }
    );
}

#[test]
fn class_bases_are_not_call_sites_and_calls_through_workspaces_resolve() {
    let (root, graph) = build(true);
    let classes = root.join("src/classes.ts");
    assert!(!graph
        .resolved_call_sites()
        .iter()
        .any(|site| site.file == classes && site.invocation == InvocationKind::Construct));

    let use_file = root.join("src/use.ts");
    let site = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.file == use_file && site.source_callee == "LibError")
        .expect("the construction is recorded");
    assert_eq!(site.invocation, InvocationKind::Construct);
    let lib = root.join("packages/lib/index.ts");
    assert_module_export(
        &site.target,
        "@fixture/lib",
        "LibError",
        Some((&lib, "LibError")),
    );
}

#[test]
fn class_bases_are_empty_without_call_analysis() {
    let (_, graph) = build(false);
    assert!(graph.resolved_class_bases().is_empty());
}

fn call(is_callback: bool, invocation: InvocationKind, owned: bool) -> FunctionCall {
    FunctionCall {
        caller: owned.then(|| "Child".to_string()),
        caller_id: owned.then_some(CallableId(7)),
        syntactic_caller: None,
        callee: "Base".to_string(),
        line: 1,
        offset: 0,
        is_callback,
        invocation,
        target_identity: CallTargetIdentity::Unknown,
        callee_binding_scope: None,
        static_arg: None,
        static_cwd: None,
    }
}

#[test]
fn only_owned_callback_constructs_are_class_bases() {
    assert_eq!(
        class_base_owner(&call(true, InvocationKind::Construct, true)),
        Some((CallableId(7), "Child"))
    );
    assert_eq!(
        class_base_owner(&call(false, InvocationKind::Construct, true)),
        None
    );
    assert_eq!(
        class_base_owner(&call(true, InvocationKind::Call, true)),
        None
    );
    assert_eq!(
        class_base_owner(&call(true, InvocationKind::Construct, false)),
        None
    );
}
