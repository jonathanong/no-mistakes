use super::super::modules;
use super::support::{catalog, world};
use crate::codebase::dependencies::graph::{merge_import_target, ImportTarget};

#[test]
fn declare_module_skips_a_truncated_declaration() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/ambient-module-spelling/truncated.d.ts"
    ));
    assert!(modules::declare_module_specifiers(source).is_empty());
}

#[test]
fn fragment_resolver_distinguishes_external_and_missing_project_imports() {
    use super::super::import_resolution::{FragmentImportResolution, FragmentResolver};
    use crate::codebase::analysis_session::AnalysisSession;
    use crate::codebase::ts_resolver::TsConfigCatalog;
    use crate::codebase::workspace_resolution::IndexedWorkspaceMap;
    use std::path::Path;

    let root = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/ambient-module-spelling"
    ));
    let importer = root.join("consumer.ts");
    let files = [importer.clone()];
    let session = AnalysisSession::new(None);
    let tsconfigs = TsConfigCatalog::from_visible(root, &[], &files);
    let workspace = IndexedWorkspaceMap::default();
    let resolution = FragmentImportResolution::new(&session, &tsconfigs, &workspace, &files);

    assert_eq!(resolution.resolve("nospace-widget", &importer, false), None);
    assert_eq!(resolution.resolve("nospace-widget", &importer, true), None);
    assert_eq!(resolution.resolve("./missing", &importer, true), Some(None));
}

#[test]
fn ambiguous_target_can_be_probed_without_recording_a_gap() {
    let mut world = world();
    let escape = world.escape.clone();
    merge_import_target(
        &mut world.graph.import_targets,
        &escape,
        "ambiguous-widget",
        ImportTarget::Ambiguous,
    );
    let mut catalog = catalog(&world);
    assert_eq!(
        modules::module_lookup(
            Some(&world.graph),
            &catalog,
            &escape,
            "ambiguous-widget",
            false
        ),
        None
    );
    assert!(catalog.take_unresolved().is_empty());
}
