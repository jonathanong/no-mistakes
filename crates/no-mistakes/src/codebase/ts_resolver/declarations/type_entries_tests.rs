use super::package_entry;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::{normalize_path, TsConfig, TsConfigCatalog};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct BareLookup;

impl TsFactLookup for BareLookup {
    fn get_ts_facts(&self, _path: &Path) -> Option<&TsFileFacts> {
        None
    }
}

fn fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/type-root-entries"),
    )
}

fn entry(root: &Path, name: &str) -> Option<PathBuf> {
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    package_entry(&root.join(name), &sources)
}

#[test]
fn type_root_package_entry_follows_manifest_types_then_index_files() {
    let root = fixture();
    let expect = |dir: &str, file: &str| {
        assert_eq!(
            entry(&root, dir),
            Some(normalize_path(&root.join(dir).join(file))),
            "{dir}"
        );
    };
    expect("types-field", "worker.d.ts");
    expect("typings-field", "worker.d.ts");
    expect("index-after-manifest", "index.d.ts");
    expect("non-string-types", "index.d.ts");
    expect("bad-manifest", "index.d.ts");
    expect("index-mts", "index.d.mts");
    expect("index-cts", "index.d.cts");
    assert_eq!(entry(&root, "missing-types"), None);
    assert_eq!(entry(&root, "no-entry"), None);
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    assert_eq!(package_entry(&root.join("not-a-dir.txt"), &sources), None);
}

fn sources() -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(&[])))
}

fn selection_root() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/type-entry-selection"),
    )
}

fn forced(root: &Path, config: PathBuf) -> TsConfigCatalog {
    TsConfigCatalog::forced(
        root,
        TsConfig {
            dir: root.to_path_buf(),
            paths_dir: root.to_path_buf(),
            ..TsConfig::default()
        },
        Some(config),
    )
}

#[test]
fn resolve_name_reads_absolute_files_and_explicit_type_roots() {
    let root = fixture();
    let sources = sources();
    let absolute = root.join("types-field/worker.d.ts");
    assert_eq!(
        super::resolve_name(
            absolute.to_str().expect("utf8"),
            &root,
            None,
            &root,
            &root,
            &sources,
        ),
        Some(normalize_path(&absolute))
    );
    let missing = root.join("missing-absolute.d.ts");
    assert_eq!(
        super::resolve_name(
            missing.to_str().expect("utf8"),
            &root,
            None,
            &root,
            &root,
            &sources,
        ),
        None
    );
    // The first root has no entry, so resolution continues into the manifest root.
    let roots = vec![root.join("no-entry"), root.clone()];
    assert_eq!(
        super::resolve_name("types-field", &root, Some(&roots), &root, &root, &sources),
        Some(normalize_path(&root.join("types-field/worker.d.ts")))
    );
}

#[test]
fn reusable_facts_skip_lookups_that_omit_calls() {
    let lookup = BareLookup;
    assert!(super::reusable_facts(Some(&lookup), Path::new("missing.d.ts")).is_none());
    assert!(super::reusable_facts(None, Path::new("missing.d.ts")).is_none());
}

#[test]
fn visible_modules_skip_broken_configs_and_repeated_names() {
    let root = selection_root();
    let sources = sources();
    let session = AnalysisSession::disabled();
    let broken = forced(&root, root.join("broken.json"));
    let skipped = super::visible_modules(session.as_ref(), &root, &sources, Some(&broken), None);
    assert!(matches!(
        skipped.class(
            "spec:present",
            &root.join("importer.ts"),
            "Present",
            Some(&broken)
        ),
        super::super::ambient::ClassLookup::Missing
    ));

    let unset = forced(&root, root.join("no-types.json"));
    let empty = super::visible_modules(session.as_ref(), &root, &sources, Some(&unset), None);
    assert!(matches!(
        empty.class(
            "spec:present",
            &root.join("importer.ts"),
            "Present",
            Some(&unset)
        ),
        super::super::ambient::ClassLookup::Missing
    ));

    let catalog = forced(&root, root.join("tsconfig.json"));
    let modules = super::visible_modules(session.as_ref(), &root, &sources, Some(&catalog), None);
    assert!(matches!(
        modules.class(
            "spec:present",
            &root.join("importer.ts"),
            "Present",
            Some(&catalog)
        ),
        super::super::ambient::ClassLookup::Unique(_, _)
    ));
}
