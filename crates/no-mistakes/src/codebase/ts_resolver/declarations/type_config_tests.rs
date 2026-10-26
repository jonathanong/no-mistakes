use super::inherited;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn root() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/type-config-shapes"),
    )
}

fn sources() -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(&[])))
}

fn load(name: &str) -> super::PartialTypes {
    let path = root().join(name);
    inherited(&sources(), &path, &mut Vec::new()).unwrap_or_else(|| panic!("{name} did not load"))
}

#[test]
fn inherited_types_follow_relative_configs_and_skip_packages() {
    let root = root();
    assert_eq!(load("cycle-a.json").names.unwrap().0, vec!["from-a"]);
    assert_eq!(load("cycle-b.json").names.unwrap().0, vec!["from-b"]);

    assert_eq!(load("package-extends.json").names.unwrap().0, vec!["kept"]);

    let odd = load("non-array.json");
    assert!(odd.names.unwrap().0.is_empty());
    assert_eq!(odd.type_roots, Some(Vec::new()));

    assert_eq!(
        load("roots.json").type_roots.unwrap(),
        vec![
            normalize_path(&root.join("relative-root")),
            normalize_path(Path::new("/tmp/nm-abs-type-root")),
        ]
    );

    let layered = load("child-extends-array.json");
    assert_eq!(layered.names.unwrap().0, vec!["from-child"]);
    assert_eq!(
        layered.type_roots.unwrap(),
        vec![normalize_path(&root.join("from-parent"))]
    );

    let from_dir = load("config-dir.json");
    assert_eq!(from_dir.names.unwrap().0, vec!["from-config-dir"]);
    assert_eq!(
        from_dir.type_roots.unwrap(),
        vec![normalize_path(&root.join("from-parent"))]
    );
}
