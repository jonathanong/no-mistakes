use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::queries::resolve_check::batch_report_from_prepared_facts;
use crate::codebase::ts_source::facts::{TsFactMap, TsFactPlan, TsFileFacts};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::PathBuf;
use std::sync::Arc;

pub const VOUCHINGTON_RESOLVE_CHECK_FILE_COUNT: usize = 2_120;
const VOUCHINGTON_VISIBLE_FILE_COUNT: usize = 17_028;

/// Synthetic prepared request shaped like the Vouchington route batch: many
/// prepared resolve-check file closure from Vouchington's 429 route roots,
/// sharing its current request-visible repository universe. The route files
/// exist on disk so nearest-config lookup takes its production file branch.
/// File facts are already prepared; only the root tsconfig is read.
pub struct PreparedResolveCheckFixture {
    root: PathBuf,
    files: Vec<PathBuf>,
    facts: TsFactMap,
    visible: crate::fx::PathSet,
    source_store: SourceStore,
}

pub fn prepared_resolve_check_fixture() -> PreparedResolveCheckFixture {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/core-analysis")
        .canonicalize()
        .expect("core-analysis performance fixture should exist");
    let files = (0..VOUCHINGTON_RESOLVE_CHECK_FILE_COUNT)
        .map(|index| root.join(format!("web/app/routes/closure-{index}/page.tsx")))
        .collect::<Vec<_>>();
    let visible_paths = (0..(VOUCHINGTON_VISIBLE_FILE_COUNT - files.len() - 1))
        .map(|index| root.join(format!("web/lib/generated/module-{index}.ts")))
        .chain(files.iter().cloned())
        .chain(std::iter::once(root.join("tsconfig.json")))
        .collect::<Vec<_>>();
    let visible = visible_paths.iter().cloned().collect();
    let source_store = SourceStore::new(Arc::new(FileInventory::from_paths(&visible_paths)));
    let facts = TsFactMap::from_iter_with_plan(
        files.iter().cloned().map(|file| {
            (
                file,
                TsFileFacts {
                    imports: vec![ExtractedImport {
                        specifier: "@vouchington/shared".to_string(),
                        kind: ImportKind::Static,
                        line: 1,
                        function_scope: None,
                        function_scope_id: None,
                        side_effect_only: false,
                        re_export: false,
                        runtime_reachable: true,
                        computed: false,
                    }],
                    ..TsFileFacts::default()
                },
            )
        }),
        TsFactPlan::imports(),
    );
    PreparedResolveCheckFixture {
        root: root.clone(),
        files,
        facts,
        visible,
        source_store,
    }
}

pub fn run_prepared_resolve_check(fixture: &PreparedResolveCheckFixture) -> usize {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let report = batch_report_from_prepared_facts(
        &fixture.root,
        fixture.files.clone(),
        &fixture.facts,
        &fixture.visible,
        &fixture.source_store,
        None,
        &session,
    )
    .expect("prepared resolve-check benchmark should succeed");
    serde_json::to_value(&report).expect("batch report should serialize")["results"]
        .as_array()
        .map(Vec::len)
        .expect("batch report must contain a results array")
}

#[cfg(test)]
mod tests;
