use super::*;
use crate::codebase::dependencies::extract::{CallTargetIdentity, CallableId};

fn build(class_hierarchy: bool, calls: bool) -> (PathBuf, DepGraph) {
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
            class_hierarchy,
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
    let (root, graph) = build(true, false);
    let classes = root.join("src/classes.ts");
    let base = root.join("src/base.ts");
    let lib = root.join("packages/lib/index.ts");
    let bases = &graph.class_hierarchy().bases;
    let summary: Vec<_> = bases
        .iter()
        .map(|class| {
            (
                class.file.strip_prefix(&root).unwrap().to_str().unwrap(),
                class.class_scope.as_str(),
                class.exported,
                class.line,
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("packages/lib/index.ts", "LibError", true, 1),
            ("src/classes.ts", "Local", true, 6),
            ("src/classes.ts", "FromNamespace", true, 7),
            ("src/classes.ts", "FromWorkspace", true, 8),
            ("src/classes.ts", "SameFile", true, 9),
            ("src/classes.ts", "Builtin", true, 10),
            ("src/classes.ts", "FromPackage", true, 11),
            ("src/classes.ts", "Private", true, 15),
            ("src/classes.ts", "default", true, 18),
        ]
    );

    let target = |scope: &str| {
        &bases
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
fn only_the_class_hierarchy_follows_workspace_names_and_class_bases_are_not_call_sites() {
    let (root, graph) = build(true, true);
    let use_file = root.join("src/use.ts");
    let lib = root.join("packages/lib/index.ts");
    let liberror = |sites: &[ResolvedCallSite]| {
        sites
            .iter()
            .find(|site| site.file == use_file && site.source_callee == "LibError")
            .expect("the construction is recorded")
            .clone()
    };
    let construction = liberror(&graph.class_hierarchy().constructions);
    assert_eq!(construction.invocation, InvocationKind::Construct);
    assert_module_export(
        &construction.target,
        "@fixture/lib",
        "LibError",
        Some((&lib, "LibError")),
    );
    // The same call stays unresolved for everything that reads call sites.
    assert_module_export(
        &liberror(graph.resolved_call_sites()).target,
        "@fixture/lib",
        "LibError",
        None,
    );

    let classes = root.join("src/classes.ts");
    let constructs_in_classes = |site: &&ResolvedCallSite| {
        site.file == classes && site.invocation == InvocationKind::Construct
    };
    let hierarchy = &graph.class_hierarchy().constructions;
    assert_eq!(hierarchy.iter().filter(constructs_in_classes).count(), 0);
    let sites = graph.resolved_call_sites();
    assert_eq!(sites.iter().filter(constructs_in_classes).count(), 0);
}

#[test]
fn the_class_hierarchy_is_empty_unless_requested() {
    for calls in [false, true] {
        let (_, graph) = build(false, calls);
        assert!(graph.class_hierarchy().bases.is_empty());
        assert!(graph.class_hierarchy().constructions.is_empty());
    }
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
