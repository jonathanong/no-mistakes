use super::{
    identities_match, looks_like_repo_relative_module, path_ends_with_module, ModuleIdentity,
};
use std::path::{Path, PathBuf};

#[test]
fn wrapper_suffix_matches_path_component_with_extension() {
    assert!(path_ends_with_module(
        Path::new("/repo/src/web/app.ts"),
        "web/app"
    ));
}

#[test]
fn wrapper_suffix_matches_an_exact_repo_relative_path() {
    assert!(path_ends_with_module(Path::new("web/app.ts"), "web/app"));
}

#[test]
fn wrapper_suffix_does_not_match_inside_a_directory_name() {
    assert!(!path_ends_with_module(
        Path::new("/repo/src/not-web/app.ts"),
        "web/app"
    ));
}

#[test]
fn repo_relative_modules_require_a_slash() {
    assert!(looks_like_repo_relative_module("web/app"));
    assert!(!looks_like_repo_relative_module("app"));
    assert!(!looks_like_repo_relative_module("./web/app"));
    assert!(!looks_like_repo_relative_module("@app/web"));
}

#[test]
fn unresolved_repo_relative_wrappers_match_imported_path_suffixes() {
    let imported = Some(ModuleIdentity::Path(PathBuf::from("/repo/src/web/app.ts")));
    assert!(identities_match("web/app", None, imported.clone()));
    assert!(!identities_match("web/other", None, imported.clone()));
    assert!(!identities_match("./web/app", None, imported));
    assert!(!identities_match(
        "web/app",
        None,
        Some(ModuleIdentity::External("web/app".to_string())),
    ));
    assert!(identities_match(
        "web/app",
        Some(ModuleIdentity::External("web/app".to_string())),
        Some(ModuleIdentity::Path(PathBuf::from("/repo/src/web/app.ts"))),
    ));
    assert!(!identities_match(
        "web/app",
        Some(ModuleIdentity::External("web/app".to_string())),
        Some(ModuleIdentity::Path(PathBuf::from("/repo/src/other.ts"))),
    ));
    assert!(!identities_match(
        "src/helpers/locator",
        Some(ModuleIdentity::Path(PathBuf::from(
            "/repo/src/helpers/locator.ts"
        ))),
        Some(ModuleIdentity::Path(PathBuf::from(
            "/repo/packages/web/src/helpers/locator.ts"
        ))),
    ));
}
