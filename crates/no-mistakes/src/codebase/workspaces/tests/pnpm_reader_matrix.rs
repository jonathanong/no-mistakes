use super::super::{resolve_pnpm_include_at_root, PackageGlobs, ReleaseAgeExclude};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::{path::PathBuf, sync::Arc};

#[test]
fn saved_includes_reserve_unreadable_grants_and_keep_valid_rebased_metadata() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/pnpm-include-reader-matrix/fixture");
    for case in ["failures", "invalid-map", "root-only", "valid"] {
        let root = crate::codebase::ts_resolver::normalize_path(&base.join(case));
        let files = crate::codebase::ts_source::discover_files(&root, &[]);
        let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
        let include = resolve_pnpm_include_at_root(&root, &sources)
            .unwrap()
            .unwrap();
        match case {
            "failures" => {
                assert_eq!(include.load_errors.len(), 8);
                for expected in [
                    "invalid grant",
                    "not a local YAML file",
                    "escapes the workspace root",
                    "could not be read",
                    "not valid YAML",
                    "must be a mapping",
                ] {
                    assert!(
                        include
                            .load_errors
                            .iter()
                            .any(|error| error.message.contains(expected)),
                        "{case}: missing {expected}"
                    );
                }
                assert!(matches!(include.packages, PackageGlobs::Conflict));
                assert!(include
                    .load_errors
                    .iter()
                    .any(|error| error.affects("minimumReleaseAgeExclude")));
                assert_eq!(
                    include.included_override_paths,
                    [root.join("settings/policy.yml")]
                );
            }
            "invalid-map" => {
                assert_eq!(include.load_errors.len(), 1);
                assert_eq!(
                    include.load_errors[0].message,
                    "pnpm include must be a mapping of local YAML files"
                );
                assert!(include.load_errors[0].affects("packages"));
                assert!(include.load_errors[0].affects("overrides"));
                assert!(
                    matches!(include.packages, PackageGlobs::List(ref values) if values.is_empty())
                );
            }
            "root-only" => {
                assert!(include.load_errors.is_empty());
                assert!(
                    matches!(include.packages, PackageGlobs::List(ref values) if values == &["apps/*"])
                );
                assert!(matches!(
                    include.release_age_exclude,
                    ReleaseAgeExclude::FromRoot
                ));
            }
            "valid" => {
                assert!(include.load_errors.is_empty());
                assert!(include.duplicates.is_empty());
                assert!(
                    matches!(include.packages, PackageGlobs::List(ref values) if values == &["settings/nested/*"])
                );
                assert!(
                    matches!(include.release_age_exclude, ReleaseAgeExclude::Included(ref values) if values == &[serde_yaml::Value::String("example".into())])
                );
                assert_eq!(
                    include.included_override_paths,
                    [root.join("settings/security.yml")]
                );
                let catalog = include.catalogs.get("default", "example").unwrap();
                assert_eq!(catalog.specifier, "1.0.0");
                assert_eq!(catalog.base_dir, root.join("settings"));
            }
            _ => unreachable!(),
        }
        let reads = sources.physical_read_count();
        resolve_pnpm_include_at_root(&root, &sources)
            .unwrap()
            .unwrap();
        assert_eq!(
            sources.physical_read_count(),
            reads,
            "{case}: request source reuse"
        );
    }
}
