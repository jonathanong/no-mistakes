use super::*;
use crate::codebase::queries::shared::resolve_target;
use crate::codebase::ts_resolver::normalize_path;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture"),
    )
}

#[test]
fn direct_and_file_importer_projections_preserve_their_distinct_meaning() {
    let root = fixture_root();
    let target = resolve_target(Path::new("util.ts"), Some(&root), None).unwrap();
    let analysis = build_reverse_analysis(&target).unwrap();

    // A symbol projection only returns concrete imports of that symbol, while
    // a file projection is the union of the file's indexed consumers.
    assert_eq!(
        direct_importer_paths(&analysis.index, &target.abs_file, "used", &target.root),
        vec![
            "barrel.ts".to_string(),
            "broken.ts".to_string(),
            "consumer.ts".to_string(),
        ]
    );
    assert!(
        direct_importer_paths(&analysis.index, &target.abs_file, "removed", &target.root)
            .is_empty()
    );
    assert_eq!(
        super::importers::file_importer_paths(&analysis.index, &target.abs_file, &target.root),
        vec![
            "barrel.ts".to_string(),
            "broken.ts".to_string(),
            "consumer.ts".to_string(),
        ]
    );
}

#[test]
fn symbols_distinguishes_recovered_and_incomplete_prepared_facts() {
    let root = fixture_root();
    let target = resolve_target(Path::new("util.ts"), Some(&root), None).unwrap();

    let mut missing_facts = build_reverse_analysis(&target).unwrap();
    missing_facts.facts.remove(&target.abs_file);
    let error = missing_facts.symbols(&target).unwrap_err().to_string();
    assert!(error.contains("missing facts for"), "{error}");
    assert!(error.contains("util.ts"), "{error}");

    let mut recovered_diagnostic = build_reverse_analysis(&target).unwrap();
    recovered_diagnostic
        .facts
        .get_mut(&target.abs_file)
        .unwrap()
        .parse_error = Some("fixture parser diagnostic".to_string());
    assert!(recovered_diagnostic.symbols(&target).is_ok());

    let mut fatal_parse_error = build_reverse_analysis(&target).unwrap();
    let facts = fatal_parse_error.facts.get_mut(&target.abs_file).unwrap();
    facts.parse_error = Some("fixture parser diagnostic".to_string());
    facts.fatal_parse_error = true;
    let error = fatal_parse_error.symbols(&target).unwrap_err().to_string();
    assert!(error.contains("extracting symbols from"), "{error}");
    assert!(error.contains("fixture parser diagnostic"), "{error}");

    let mut fatal_without_diagnostic = build_reverse_analysis(&target).unwrap();
    fatal_without_diagnostic
        .facts
        .get_mut(&target.abs_file)
        .unwrap()
        .fatal_parse_error = true;
    let error = fatal_without_diagnostic
        .symbols(&target)
        .unwrap_err()
        .to_string();
    assert!(error.contains("fatal parser failure"), "{error}");
    assert!(error.contains("util.ts"), "{error}");

    let mut diagnostic_without_symbols = build_reverse_analysis(&target).unwrap();
    let facts = diagnostic_without_symbols
        .facts
        .get_mut(&target.abs_file)
        .unwrap();
    facts.parse_error = Some("fixture parser diagnostic".to_string());
    facts.symbols = None;
    let error = diagnostic_without_symbols
        .symbols(&target)
        .unwrap_err()
        .to_string();
    assert!(error.contains("extracting symbols from"), "{error}");
    assert!(error.contains("fixture parser diagnostic"), "{error}");

    let mut missing_symbols = build_reverse_analysis(&target).unwrap();
    missing_symbols
        .facts
        .get_mut(&target.abs_file)
        .unwrap()
        .symbols = None;
    let error = missing_symbols.symbols(&target).unwrap_err().to_string();
    assert!(error.contains("missing symbols for"), "{error}");
    assert!(error.contains("util.ts"), "{error}");
}
