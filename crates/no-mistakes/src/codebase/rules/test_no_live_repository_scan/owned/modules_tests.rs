use super::{file_stem, specifier_stem};
use std::path::Path;

#[test]
fn specifiers_reduce_to_the_module_name() {
    assert_eq!(specifier_stem("../src/dir").as_deref(), Some("dir"));
    assert_eq!(specifier_stem("./dir.js").as_deref(), Some("dir"));
    assert_eq!(specifier_stem("./lib/index.ts").as_deref(), Some("lib"));
    assert_eq!(specifier_stem("./lib/").as_deref(), Some("lib"));
    assert_eq!(specifier_stem("lodash").as_deref(), Some("lodash"));
}

#[test]
fn bare_relative_specifiers_have_no_name() {
    assert_eq!(specifier_stem("."), None);
    assert_eq!(specifier_stem(".."), None);
    assert_eq!(specifier_stem("./index"), None);
    assert_eq!(specifier_stem(""), None);
}

#[test]
fn files_reduce_to_the_same_name() {
    assert_eq!(
        file_stem(Path::new("/r/src/dir.ts")).as_deref(),
        Some("dir")
    );
    assert_eq!(
        file_stem(Path::new("/r/src/lib/index.ts")).as_deref(),
        Some("lib")
    );
    assert_eq!(file_stem(Path::new("/index.ts")), None);
}
