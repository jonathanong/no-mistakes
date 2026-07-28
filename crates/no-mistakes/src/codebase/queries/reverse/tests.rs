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
        file_importer_paths(&analysis.index, &target.abs_file, &target.root),
        vec![
            "barrel.ts".to_string(),
            "broken.ts".to_string(),
            "consumer.ts".to_string(),
        ]
    );
}
