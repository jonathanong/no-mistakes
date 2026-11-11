use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::fact_panic::tests::with_injected_panic;
use crate::codebase::ts_source::{FileInventory, SourceStore};

#[test]
fn panicked_variants_preserve_declared_source_without_ast_facts_or_extra_reads() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/symbols/signature-impact-encoding/tolerated")
        .canonicalize()
        .unwrap();
    let path = root.join("comment.mts");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(std::slice::from_ref(
        &path,
    ))));
    assert!(sources.read_path(&path).is_err());
    let source = sources.read_source_path(&path).unwrap();
    assert!(source.contains('\u{fffd}'));
    let plans = [
        CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                source: true,
                ..Default::default()
            },
            ..Default::default()
        },
        CheckFactPlan {
            source: true,
            ..Default::default()
        },
        CheckFactPlan::default(),
    ];
    let variants = plans
        .iter()
        .map(|plan| CheckFactVariant {
            root: &root,
            plan,
            playwright: None,
            collect_tests: false,
        })
        .collect::<Vec<_>>();
    let session = AnalysisSession::disabled();
    crate::ast::begin_parse_count(&root);
    let facts = with_injected_panic(&path, || {
        super::super::collect_file_fact_variants_from_source_with_session(
            &session,
            &path,
            Arc::clone(&source),
            &variants,
        )
    });
    assert!(crate::ast::finish_parse_count(&root).is_empty());
    for (index, facts) in facts.into_iter().enumerate() {
        let facts = facts.unwrap();
        assert!(facts.fact_extraction_panicked);
        assert!(!facts.parsed);
        assert!(facts.ts.operational_error.is_some());
        assert!(facts.ts.imports.is_empty());
        assert!(facts.ts.symbols.is_none());
        assert!(facts.ts.function_calls.is_empty());
        assert_eq!(facts.source.is_some(), index == 1);
        if index < 2 {
            assert!(Arc::ptr_eq(facts.ts.source.as_ref().unwrap(), &source));
        } else {
            assert!(facts.ts.source.is_none());
        }
    }
    let facts = with_injected_panic(&path, || {
        super::super::super::collect_file_facts_with_request(
            session.as_ref().into(),
            &root,
            &path,
            &plans[0],
            None,
            &sources,
            false,
        )
        .unwrap()
    });
    assert!(Arc::ptr_eq(facts.ts.source.as_ref().unwrap(), &source));
    assert!(facts.source.is_none());
    assert_eq!(sources.physical_read_count(), 1);
}
