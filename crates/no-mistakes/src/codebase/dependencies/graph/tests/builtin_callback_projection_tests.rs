use super::*;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactMap, TsFactPlan, TsFileFacts};

struct CallbackFacts<'a> {
    base: &'a TsFactMap,
    failed: Option<(PathBuf, TsFileFacts)>,
}

impl TsFactLookup for CallbackFacts<'_> {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.failed
            .as_ref()
            .filter(|(failed, _)| failed == path)
            .map(|(_, facts)| facts)
            .or_else(|| self.base.get(path))
    }

    fn covers_ts_fact_plan(&self, plan: TsFactPlan) -> bool {
        self.base.covers_ts_fact_plan(plan)
    }
}

fn fixture_graph(fail_import: bool) -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins"),
    );
    let files = GraphFiles::discover(&root);
    let plan = GraphBuildPlan {
        calls: true,
        imports: true,
        assertion_owners: true,
        ..GraphBuildPlan::default()
    };
    let base = collect_ts_facts(files.indexable(), plan.ts_fact_plan());
    let failed = fail_import.then(|| {
        let path = root.join("src/failed-owner.mts");
        let mut facts = base.get(&path).unwrap().clone();
        facts.parse_error = Some("injected saved source parse failure".into());
        (path, facts)
    });
    let facts = CallbackFacts {
        base: &base,
        failed,
    };
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        plan,
        &files,
        None,
        Some(&facts),
    )
    .unwrap();
    (root, graph)
}

#[test]
fn native_callback_projection_keeps_file_roots_and_external_alias_metadata() {
    let (root, graph) = fixture_graph(false);
    let source = root.join("src/projection-branches.mts");
    let owner = root.join("src/owner.mts");
    assert!(graph
        .dependencies_of_node(&NodeId::file(&source))
        .into_iter()
        .flatten()
        .any(|(target, kind)| *kind == EdgeKind::AssertionOwner
            && target.as_file() == Some(owner.as_path())));
    let opaque = graph
        .assertion_call_sites()
        .iter()
        .find(|site| site.file == source && site.source_callee == "externalCallback")
        .unwrap();
    assert!(graph.opaque_external_member_call(opaque), "{opaque:?}");
}

#[test]
fn native_callback_projection_retains_failed_import_prerequisite() {
    let (root, graph) = fixture_graph(true);
    let source = root.join("src/projection-branches.mts");
    let failed = root.join("src/failed-owner.mts");
    let callback = graph
        .assertion_call_sites()
        .iter()
        .find(|site| site.file == source && site.source_callee == "failedOwner")
        .unwrap();
    assert!(
        graph
            .failed_call_sources(callback)
            .any(|path| path == &failed),
        "{callback:?}"
    );
}

#[test]
fn missing_prepared_inline_scope_does_not_invent_callback_identity() {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let path = root.join("src/continuations.mts");
    let plan = TsFactPlan {
        function_calls: true,
        value_flow: true,
        call_eligibility: true,
        ..TsFactPlan::default()
    };
    let facts = collect_ts_facts(std::slice::from_ref(&path), plan);
    let mut file = facts.get(&path).unwrap().clone();
    let id = file
        .value_flow
        .calls
        .iter()
        .find_map(|call| {
            (call.source_callee.ends_with(".finally"))
                .then(|| call.arguments.first())
                .flatten()
                .and_then(|argument| match argument {
                    crate::codebase::dependencies::extract::ValueExpression::Callable(id) => {
                        Some(*id)
                    }
                    _ => None,
                })
        })
        .expect("saved inline Promise callback");
    file.callable_scope_ids
        .retain(|(candidate, _)| *candidate != id);
    let mut index = CallableFileIndex::from_facts(&file);
    index.prepare_builtin_callbacks(&file);
    assert!(!index
        .values
        .builtin_callbacks
        .iter()
        .any(|call| call.offset == id.0));
}

#[test]
fn storage_forwarded_arguments_refine_only_assertion_ownership() {
    let (root, graph) = fixture_graph(false);
    let source = root.join("src/storage-forwarded.mts");
    let owner = root.join("src/owner.mts");
    for caller in ["invokeRun", "invokeExit"] {
        let projected = graph
            .assertion_call_sites()
            .iter()
            .find(|site| {
                site.file == source
                    && site.caller.as_deref() == Some(caller)
                    && site.source_callee == "fn"
            })
            .expect("forwarded native callback argument resolves nested invocation");
        assert!(
            projected
                .target_nodes()
                .any(|target| target.as_file() == Some(owner.as_path())),
            "{projected:?}"
        );
        let ordinary = graph
            .resolved_call_sites()
            .iter()
            .find(|site| {
                site.file == source
                    && site.caller.as_deref() == Some(caller)
                    && site.source_callee == "fn"
            })
            .unwrap();
        assert!(ordinary.target_node.is_none(), "{ordinary:?}");
    }
    for caller in [
        "conflicting",
        "invokeSpread",
        "invokeMissing",
        "invokePromise",
        "invokeArray",
        "invokeTimer",
    ] {
        assert!(!graph.assertion_call_sites().iter().any(|site| {
            site.file == source
                && site.caller.as_deref() == Some(caller)
                && site.source_callee == "fn"
                && site.target_node.is_some()
        }));
    }
}
