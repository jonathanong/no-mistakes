use super::target_imports;
use crate::codebase::queries::shared::resolve_targets;
use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
use std::path::PathBuf;

fn target() -> crate::codebase::queries::shared::Target {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/queries/fixture");
    resolve_targets(&[PathBuf::from("consumer.ts")], Some(&root), None)
        .unwrap()
        .remove(0)
}

#[test]
fn target_imports_propagates_prepared_fact_failures() {
    let target = target();
    let read_error = TsFactMap::from([(
        target.abs_file.clone(),
        TsFileFacts {
            operational_error: Some("failed to read consumer.ts: invalid UTF-8".to_string()),
            ..TsFileFacts::default()
        },
    )]);
    assert!(target_imports(&target, &read_error)
        .unwrap_err()
        .to_string()
        .contains("failed to read"));

    let fatal_parse = TsFactMap::from([(
        target.abs_file.clone(),
        TsFileFacts {
            parse_error: Some("parser panicked".to_string()),
            fatal_parse_error: true,
            ..TsFileFacts::default()
        },
    )]);
    assert!(target_imports(&target, &fatal_parse)
        .unwrap_err()
        .to_string()
        .contains("failed to parse"));

    let missing = TsFactMap::default();
    assert!(target_imports(&target, &missing)
        .unwrap_err()
        .to_string()
        .contains("missing facts"));
}
