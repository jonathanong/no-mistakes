use super::super::resolve_pnpm_include_at_root;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::{path::PathBuf, sync::Arc};

#[test]
fn catalog_values_filter_invalid_entries_and_unreadable_grants_keep_ownership() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/pnpm-include-reader-matrix/fixture");
    for case in ["catalog-values", "catalog-reserved"] {
        let root = crate::codebase::ts_resolver::normalize_path(&base.join(case));
        let files = crate::codebase::ts_source::discover_files(&root, &[]);
        let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
        let include = resolve_pnpm_include_at_root(&root, &sources)
            .unwrap()
            .unwrap();
        if case == "catalog-values" {
            assert!(include.load_errors.is_empty());
            assert!(include.duplicates.is_empty());
            for (catalog, package, expected) in [
                ("default", "root", "1.0.0"),
                ("default", "duplicate", "1.0.0"),
                ("default", "other", "3.0.0"),
                ("named", "valid", "4.0.0"),
            ] {
                let value = include.catalogs.get(catalog, package).unwrap();
                assert_eq!(value.specifier, expected);
                assert_eq!(value.base_dir, root);
            }
            for (catalog, package) in [
                ("default", "invalid"),
                ("default", "42"),
                ("named", "invalid"),
                ("nonmapping", "valid"),
                ("12", "ignored"),
            ] {
                assert!(
                    include.catalogs.get(catalog, package).is_none(),
                    "{catalog}/{package}"
                );
            }
        } else {
            assert_eq!(include.load_errors.len(), 1);
            assert!(include.load_errors[0].message.contains("could not be read"));
            assert!(include.load_errors[0].affects("catalogs"));
            assert_eq!(include.duplicates, ["catalogs", "catalogs.named"]);
            assert!(include.catalogs.get("named", "valid").is_none());
        }
    }
}
