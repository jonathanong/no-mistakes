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

#[test]
fn quoted_import_lines_keep_string_specifiers_and_unicode_bounds_names() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends/language-symbols/lines");
    let quoted = root.join("quoted_import.go");
    let unicode = root.join("unicode_name.py");
    let sources = store(&quoted);
    assert_eq!(
        first_import_line(&sources, &quoted, "example.com/foo/bar"),
        Some(4)
    );
    assert_eq!(
        first_import_line(&sources, &quoted, "example.com/foo"),
        Some(5)
    );
    assert_eq!(
        first_import_line(&sources, &quoted, "example.com/missing"),
        None
    );
    let sources = store(&unicode);
    assert_eq!(
        (
            first_name_line(&sources, &unicode, "Greeter"),
            first_name_line(&sources, &unicode, "_Greeter"),
            first_name_line(&sources, &unicode, "Greeter_id"),
        ),
        (Some(3), Some(2), Some(4))
    );
    let rust = root.join("rust_lifetime.rs");
    let sources = store(&rust);
    assert_eq!(first_name_line(&sources, &rust, "Greeter"), Some(1));
}
