use super::*;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use crate::codebase::workspaces::load_indexed_from_source_store;

struct Fixture {
    root: PathBuf,
    workspace: IndexedWorkspaceMap,
    visible: PathSet,
    files: FileIdMap<CheckFileFacts>,
}

impl Fixture {
    fn new(executors: &[&str]) -> Self {
        let root = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-require-query-annotation/fixture/workspace-fallback-unit",
        ));
        let paths = discover_visible_paths(&root);
        let inventory = Arc::new(FileInventory::from_paths(&paths));
        let sources = SourceStore::new(Arc::clone(&inventory));
        let workspace = load_indexed_from_source_store(&root, &sources).expect("workspace");
        let mut files = FileIdMap::with_inventory(inventory);
        files.insert(
            root.join("caller.mts"),
            CheckFileFacts {
                query_annotation: executors
                    .iter()
                    .map(|name| {
                        (
                            EmbeddedSqlOptions::configured(name, &[]),
                            Default::default(),
                        )
                    })
                    .collect(),
                ..CheckFileFacts::default()
            },
        );
        Self {
            root,
            workspace,
            visible: paths.into_iter().collect(),
            files,
        }
    }

    fn fallback<'a>(&'a self, importers: Option<&'a PathSet>) -> WorkspaceFallback<'a> {
        WorkspaceFallback::new(&self.workspace, &self.visible, importers, &self.files)
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }
}

#[test]
fn primary_hits_keep_precedence_and_invisible_hits_veto_fallback() {
    let fixture = Fixture::new(&[]);
    let from = fixture.path("caller.mts");
    let primary = fixture.path("other.mts");
    let fallback = fixture.fallback(None);
    assert_eq!(
        fallback.resolve("@fixture/db", &from, |_, _| Some(primary.clone())),
        Some(primary)
    );
    assert_eq!(
        fallback.resolve("@fixture/db", &from, |_, _| Some(
            fixture.path("missing.mts")
        )),
        None
    );
}

#[test]
fn configured_importers_are_exact_files_and_none_preserves_legacy() {
    let fixture = Fixture::new(&[]);
    let from = fixture.path("caller.mts");
    let selected = PathSet::from_iter([from.clone()]);
    let fallback = fixture.fallback(Some(&selected));
    assert_eq!(
        fallback.resolve("@fixture/db", &from, |_, _| None),
        Some(fixture.path("packages/db/index.mts"))
    );
    assert_eq!(
        fallback.resolve("@fixture/db", &fixture.path("other.mts"), |_, _| None),
        None
    );
    assert_eq!(
        fallback.resolve("@fixture/missing", &from, |_, _| None),
        None
    );
    assert!(fixture
        .fallback(None)
        .resolve("@fixture/db-other", &from, |_, _| None)
        .is_some());
}

#[test]
fn executor_exception_matches_package_identity_and_subpaths() {
    let fixture = Fixture::new(&["", "@fixture/db", "@fixture/db"]);
    let outside = PathSet::default();
    let fallback = fixture.fallback(Some(&outside));
    let from = fixture.path("caller.mts");
    let expected = Some(fixture.path("packages/db/index.mts"));
    assert_eq!(
        fallback.resolve("@fixture/db", &from, |_, _| None),
        expected
    );
    assert_eq!(
        fallback.resolve("@fixture/db/entry", &from, |_, _| None),
        expected
    );
    assert_eq!(
        fallback.resolve("@fixture/db-other", &from, |_, _| None),
        None
    );
    let empty = Fixture::new(&[""]);
    assert_eq!(
        empty
            .fallback(Some(&outside))
            .resolve("@fixture/db", &from, |_, _| None),
        None
    );
    let missing = Fixture::new(&["@fixture/missing"]);
    assert_eq!(
        missing
            .fallback(Some(&outside))
            .resolve("@fixture/db", &from, |_, _| None),
        None
    );
}

#[test]
fn executor_alias_roots_follow_each_importing_context() {
    let fixture = Fixture::new(&["@configured/alias"]);
    let outside = PathSet::default();
    let fallback = fixture.fallback(Some(&outside));
    let first = fixture.path("caller.mts");
    let second = fixture.path("other.mts");
    let primary = |specifier: &str, from: &Path| {
        (specifier == "@configured/alias").then(|| {
            fixture.path(if from == first {
                "packages/db/index.mts"
            } else {
                "packages/db-other/index.mts"
            })
        })
    };
    assert!(fallback.resolve("@fixture/db", &first, primary).is_some());
    assert_eq!(fallback.resolve("@fixture/db", &second, primary), None);
    assert!(fallback
        .resolve("@fixture/db-other", &second, primary)
        .is_some());
}

mod observers;
