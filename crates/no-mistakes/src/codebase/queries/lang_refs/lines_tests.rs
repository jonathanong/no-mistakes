use super::lines::{first_import_line, first_name_line};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn store(path: &Path) -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(&[path.to_path_buf()])))
}

#[test]
fn name_lines_cover_misses_segments_and_import_fallbacks() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends/language-symbols/python/caller.py");
    let sources = store(&path);
    let missing_path = Path::new("/no/such/language-symbols.py");
    assert_eq!(
        (
            first_name_line(&sources, &path, "Greeter"),
            first_name_line(&sources, &path, "NoSuchName"),
            first_name_line(&sources, missing_path, "Greeter"),
            first_name_line(&sources, &path, ""),
            first_name_line(&sources, &path, "App.Greeter"),
            first_import_line(&sources, &path, "Greeter"),
            first_import_line(&sources, &path, "pkg/Greeter"),
            first_import_line(&sources, &path, "nope"),
            first_import_line(&sources, &path, "pkg/nope"),
            first_import_line(&sources, &path, ""),
        ),
        (
            Some(1),
            None,
            None,
            None,
            Some(1),
            Some(1),
            Some(1),
            None,
            None,
            None,
        )
    );
}
