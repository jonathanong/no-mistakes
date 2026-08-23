use super::{collect_dart_facts, extract_dart_imports};
use std::path::{Path, PathBuf};

#[test]
fn package_and_relative_imports_extract() {
    let path = Path::new("/app/lib/api.dart");
    let root = Path::new("/app");
    let source = r#"
import 'package:app/user.dart';
import './user.dart';
import 'dart:async';
export 'package:app/user.dart';
part 'extra.dart';
"#;
    let imports = extract_dart_imports(source, path, Some(root), Some("app"));
    assert!(imports
        .iter()
        .any(|import| import == "package:app/user.dart"));
    assert!(imports
        .iter()
        .any(|import| import == "package:app/extra.dart"));
    assert!(!imports.iter().any(|import| import.starts_with("dart:")));
}

#[test]
fn raw_import_uris_extract() {
    let path = Path::new("/app/lib/api.dart");
    let root = Path::new("/app");
    let imports = extract_dart_imports(
        "import r'package:app/user.dart';\n",
        path,
        Some(root),
        Some("app"),
    );
    assert_eq!(imports, vec!["package:app/user.dart"]);
}

#[test]
fn import_uris_are_read_from_unmasked_strings() {
    let path = Path::new("/app/lib/api.dart");
    let root = Path::new("/app");
    let source = "import 'package:app/user.dart';\n";
    let masked = super::super::strip::mask_strings(source);
    assert!(
        !extract_dart_imports(&masked, path, Some(root), Some("app"))
            .iter()
            .any(|import| import == "package:app/user.dart")
    );
    assert_eq!(
        extract_dart_imports(source, path, Some(root), Some("app")),
        vec!["package:app/user.dart"]
    );
}

#[test]
fn dart_collects_package_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/dart-flutter-http/fixture"),
    );
    let repo = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    );
    let files = crate::codebase::ts_source::discover_visible_paths(&repo)
        .into_iter()
        .map(|path| {
            let absolute = if path.is_absolute() {
                path
            } else {
                repo.join(path)
            };
            crate::codebase::ts_resolver::normalize_path(&absolute)
        })
        .filter(|path| path.starts_with(&root))
        .collect::<Vec<_>>();
    let store = crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    let facts = collect_dart_facts(&root, &files, &[".".into()], &store);
    let api = facts
        .files
        .values()
        .find(|file| file.path.ends_with("api.dart"))
        .expect("api");
    assert!(api
        .imports
        .iter()
        .any(|import| import == "package:app/user.dart"));
    assert_eq!(api.module.as_deref(), Some("package:app/api.dart"));
}

#[test]
fn pubspec_name_accepts_quotes_and_trailing_comments() {
    let quoted = super::pubspec_name_re()
        .captures("name: \"app\"\n")
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));
    let commented = super::pubspec_name_re()
        .captures("name: app # mobile package\n")
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));
    assert_eq!(quoted.as_deref(), Some("app"));
    assert_eq!(commented.as_deref(), Some("app"));
}

#[test]
fn extension_type_declarations_are_indexed() {
    let names = super::extract_named(
        "extension type UserId(int value) {}\nextension Foo on Bar {}",
        super::dart_decl_re(),
    );
    assert!(names.contains(&"UserId".to_string()));
    assert!(names.contains(&"Foo".to_string()));
}
