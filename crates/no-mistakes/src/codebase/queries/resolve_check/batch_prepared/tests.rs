use super::batch_report_from_prepared_facts;
use crate::codebase::queries::shared::resolve_targets;
use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
use std::path::PathBuf;

#[test]
fn derived_resolve_check_propagates_prepared_fact_failures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/queries/fixture");
    let target = resolve_targets(&[PathBuf::from("consumer.ts")], Some(&root), None)
        .unwrap()
        .remove(0);
    let cases = [
        (
            TsFileFacts {
                operational_error: Some("failed to read consumer.ts".to_string()),
                ..TsFileFacts::default()
            },
            "failed to read consumer.ts",
        ),
        (
            TsFileFacts {
                parse_error: Some("parser panicked".to_string()),
                fatal_parse_error: true,
                ..TsFileFacts::default()
            },
            "parser panicked",
        ),
        (
            TsFileFacts {
                fatal_parse_error: true,
                ..TsFileFacts::default()
            },
            "parser panicked without a diagnostic",
        ),
    ];
    for (file_facts, expected) in cases {
        let facts = TsFactMap::from([(target.abs_file.clone(), file_facts)]);
        let error = batch_report_from_prepared_facts(
            &target.root,
            [target.abs_file.clone()],
            &facts,
            target.visible_files(),
            &target.sources,
            None,
            &target.session,
        )
        .err()
        .expect("prepared fact failure must abort the report");
        assert!(error.to_string().contains(expected), "{error:#}");
    }
}
