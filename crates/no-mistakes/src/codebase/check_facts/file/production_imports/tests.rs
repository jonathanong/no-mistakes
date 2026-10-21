use super::*;
use crate::codebase::dependencies::extract::{
    graph_call_recording_probe, FULL_PROJECTION_FINALIZATIONS, RESOURCE_ROOT_CONSTRUCTIONS,
};

#[test]
fn prepared_production_projection_does_not_finalize_unrequested_facts() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/production-import-projection.mts",
    );
    let source: Arc<str> = std::fs::read_to_string(&path).unwrap().into();
    let root = path.parent().unwrap();
    let plan = crate::codebase::check_facts::CheckFactPlan {
        production_import_paths: Arc::new([path.clone()].into()),
        ..Default::default()
    };
    let variants = [CheckFactVariant {
        root,
        plan: &plan,
        playwright: None,
    }];
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let mut results = vec![None];

    FULL_PROJECTION_FINALIZATIONS.with(|count| count.set(0));
    RESOURCE_ROOT_CONSTRUCTIONS.with(|count| count.set(0));
    graph_call_recording_probe::reset();
    project(&session, &path, &source, &variants, &mut results);
    assert_eq!(FULL_PROJECTION_FINALIZATIONS.with(|count| count.get()), 0);
    assert_eq!(RESOURCE_ROOT_CONSTRUCTIONS.with(|count| count.get()), 0);
    // Replacing the consumer projection with the full extractor must fail here.
    assert_eq!(graph_call_recording_probe::counts(), (0, 0));
    let imports = results[0]
        .as_ref()
        .unwrap()
        .production_imports
        .as_ref()
        .unwrap();
    assert!(!imports.is_empty());
    assert!(imports.iter().any(|item| item.runtime_reachable));
}
