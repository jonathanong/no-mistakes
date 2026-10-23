use super::*;
use crate::codebase::check_facts::FamilyPathScope;
use std::cell::Cell;

thread_local! {
    pub(super) static NORMALIZATIONS: Cell<usize> = const { Cell::new(0) };
}

macro_rules! record_scope_normalization {
    () => {
        $crate::integration_tests::config::discovery_scope::tests::NORMALIZATIONS
            .with(|count| count.set(count.get() + 1));
    };
}

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/discovery-scope/fixture"),
    )
}

fn scope(root: &Path, files: &[PathBuf], config: &str) -> (FamilyPathScope, usize) {
    let config = crate::config::v2::load_v2_config(root, Some(&root.join(config))).unwrap();
    NORMALIZATIONS.with(|count| count.set(0));
    let selected = test_discovery_scope(root, files, &config);
    (selected, NORMALIZATIONS.with(Cell::get))
}

#[test]
fn all_match_avoids_owned_normalizations_and_middle_exclusion_retains_prefix_and_tail() {
    let root = fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let (all, count) = scope(&root, &files, "all-files.yml");
    assert_eq!(all, FamilyPathScope::All);
    // Prefixes conservatively use the existing lexical-cardinality fallback.
    assert_eq!(count, if cfg!(windows) { files.len() } else { 0 });
    let excluded = root.join("helpers/dependency.ts");
    let index = files.iter().position(|path| path == &excluded).unwrap();
    assert!(index > 0 && index + 1 < files.len());
    let (selected, count) = scope(&root, &files, "middle-exclude.yml");
    let FamilyPathScope::Only(paths) = selected else {
        panic!("expected explicit subset")
    };
    assert_eq!(count, files.len() - 1);
    assert_eq!(paths.len(), files.len() - 1);
    assert!(!paths.contains(&excluded));
    assert!(paths.contains(&files[index - 1]));
    assert!(paths.contains(&files[index + 1]));
    let (empty, count) = scope(&root, &files, "empty-scope.yml");
    assert!(matches!(empty, FamilyPathScope::Only(paths) if paths.is_empty()));
    assert_eq!(count, 0);
}

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    paths: Vec<PathBuf>,
    rooted: bool,
    config: String,
    members: Option<Vec<PathBuf>>,
    normalizations: usize,
}

fn assert_cases(name: &str) {
    let root = fixture();
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap();
    for case in cases {
        let rooted = |path: PathBuf| if case.rooted { root.join(path) } else { path };
        let files = case.paths.into_iter().map(rooted).collect::<Vec<_>>();
        let (selected, count) = scope(&root, &files, &case.config);
        assert_eq!(count, case.normalizations, "{}", case.name);
        match case.members {
            Some(members) => {
                let FamilyPathScope::Only(paths) = selected else {
                    panic!("{}", case.name)
                };
                let expected = members.into_iter().map(rooted).collect::<BTreeSet<_>>();
                assert_eq!(&*paths, &expected, "{}", case.name);
            }
            None => assert_eq!(selected, FamilyPathScope::All, "{}", case.name),
        }
    }
}

#[test]
fn arbitrary_slices_preserve_normalized_cardinality() {
    assert_cases("scope-path-cases.json");
}

#[cfg(windows)]
#[test]
fn windows_prefixes_keep_the_existing_fallback() {
    assert_cases("scope-path-cases-windows.json");
}
