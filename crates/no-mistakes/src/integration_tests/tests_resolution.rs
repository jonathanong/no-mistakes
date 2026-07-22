use super::*;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests")
            .join(name)
            .join("fixture"),
    )
}

#[test]
fn direct_resolution_covers_cycles_and_import_shapes() {
    let root = fixture("coverage");
    let caller = root.join("src/source.test.ts");
    let helper = root.join("src/helpers.ts");
    let caller_key = function_key(&caller, "caller");
    let imported_key = function_key(&helper, "imported");
    let cycle_a = function_key(&caller, "cycleA");
    let cycle_b = function_key(&caller, "cycleB");
    let namespace_key = function_key(&caller, "namespaceCaller");
    let named_namespace_key = function_key(&caller, "namedNamespaceCaller");
    let imported_namespace_key = function_key(&caller, "importedNamespaceCaller");

    let mut function_index = HashMap::new();
    function_index.insert(
        caller_key.clone(),
        function_info(
            None,
            vec![types::CallTarget::Imported {
                local: "defaultLocal".into(),
            }],
        ),
    );
    function_index.insert(
        imported_key.clone(),
        function_info(Some("openai"), Vec::new()),
    );
    function_index.insert(
        cycle_a.clone(),
        function_info(None, vec![types::CallTarget::Local("cycleB".into())]),
    );
    function_index.insert(
        cycle_b.clone(),
        function_info(None, vec![types::CallTarget::Local("cycleA".into())]),
    );
    function_index.insert(
        namespace_key.clone(),
        function_info(
            None,
            vec![types::CallTarget::Namespace {
                namespace: "ns".into(),
                member: "missing".into(),
            }],
        ),
    );
    function_index.insert(
        named_namespace_key.clone(),
        function_info(
            None,
            vec![types::CallTarget::Namespace {
                namespace: "namedNs".into(),
                member: "missing".into(),
            }],
        ),
    );
    function_index.insert(
        imported_namespace_key.clone(),
        function_info(
            None,
            vec![types::CallTarget::Imported { local: "ns".into() }],
        ),
    );

    let mut caller_analysis = types::FileAnalysis::default();
    caller_analysis.imports.insert(
        "defaultLocal".into(),
        import_binding("./helpers", types::ImportedName::Default),
    );
    caller_analysis.imports.insert(
        "ns".into(),
        import_binding("./helpers", types::ImportedName::Namespace),
    );
    caller_analysis.imports.insert(
        "namedNs".into(),
        import_binding("./helpers", types::ImportedName::Named("namedCall".into())),
    );

    let mut analyses = BTreeMap::new();
    analyses.insert(caller.clone(), caller_analysis);
    analyses.insert(helper.clone(), types::FileAnalysis::default());
    let mut export_index = HashMap::new();
    export_index.insert((helper, "default".into()), imported_key);
    let tsconfig = test_support::tsconfig_without_config(&root);
    let visible_files = analyses.keys().cloned().collect();
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let import_resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        &tsconfig,
        Some(&visible_files),
        &session,
    );
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(analyses.keys().cloned());
    let resolver = resolve::ImportResolution {
        analyses: &analyses,
        export_index: &export_index,
        resolver: &import_resolver,
        remapper: &remapper,
    };

    assert_eq!(
        resolve::resolved_integrations(&caller_key, &function_index, &resolver),
        vec!["openai".to_string()]
    );
    assert!(resolve::resolved_integrations(&cycle_a, &function_index, &resolver).is_empty());
    assert!(resolve::resolved_integrations(&namespace_key, &function_index, &resolver).is_empty());
    assert!(
        resolve::resolved_integrations(&named_namespace_key, &function_index, &resolver).is_empty()
    );
    assert!(
        resolve::resolved_integrations(&imported_namespace_key, &function_index, &resolver)
            .is_empty()
    );
}

#[test]
fn pass4b_call_tracing_skips_ignored_helper_for_visible_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let files = crate::codebase::ts_source::discover_source_files(&root, &[]);
    let analyses = test_support::analyze_files(&files).unwrap();
    let function_index = resolve::build_function_index(&analyses);
    let export_index = resolve::build_export_index(&analyses);
    let visible_files = files.into_iter().collect();
    let tsconfig = test_support::tsconfig_without_config(&root);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let import_resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        &tsconfig,
        Some(&visible_files),
        &session,
    );
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(analyses.keys().cloned());
    let resolver = resolve::ImportResolution {
        analyses: &analyses,
        export_index: &export_index,
        resolver: &import_resolver,
        remapper: &remapper,
    };

    let integrations = resolve::resolved_integrations(
        &function_key(&root.join("integration/caller.ts"), "caller"),
        &function_index,
        &resolver,
    );

    assert_eq!(integrations, vec!["visible".to_string()]);
}

fn import_binding(source: &str, imported: types::ImportedName) -> types::ImportBinding {
    types::ImportBinding {
        source: source.to_string(),
        imported,
    }
}

fn function_key(file: &Path, name: &str) -> types::FunctionKey {
    types::FunctionKey {
        file: file.to_path_buf(),
        name: name.to_string(),
    }
}

fn function_info(integration: Option<&str>, calls: Vec<types::CallTarget>) -> types::FunctionInfo {
    types::FunctionInfo {
        integration: integration.map(str::to_string),
        calls,
    }
}
