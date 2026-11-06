use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts, FamilyPathScope};
use crate::codebase::ts_resolver::TsConfigCatalog;

#[test]
fn missing_prepared_queue_inputs_are_failures_not_clean_negative_reports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/basic/fixture")
        .canonicalize()
        .unwrap();
    let file = root.join("enqueue.ts");
    assert!(file.exists());
    for missing_file in [true, false] {
        let mut shared = CheckFactMap {
            files: vec![file.clone()],
            queue_paths: Some(FamilyPathScope::All),
            ..Default::default()
        };
        if !missing_file {
            shared
                .ts
                .insert(file.clone(), CheckFileFacts::default().into());
        }
        let catalog = TsConfigCatalog::forced(&root, Default::default(), None);
        let session = AnalysisSession::disabled();
        let legacy = analyze(&root, &catalog, &[], &shared, &session).unwrap_err();
        assert!(legacy.to_string().contains("missing prepared queue"));
        assert!(session.check_failures().is_empty());
        session.enable_partial_checks();
        let report = analyze(&root, &catalog, &[], &shared, &session).unwrap();
        assert!(report.check.is_empty());
        let failures = session.check_failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].consumer, "queues");
        assert_eq!(failures[0].file.as_ref(), Some(&file));
        assert!(failures[0].message.contains(if missing_file {
            "missing prepared queue source facts"
        } else {
            "missing prepared queue facts"
        }));
    }
}
