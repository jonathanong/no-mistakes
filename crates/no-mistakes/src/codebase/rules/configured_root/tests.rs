use super::*;
use std::path::{Path, PathBuf};

#[test]
fn dot_repository_root_still_rejects_a_parent() {
    assert!(escapes_repository(Path::new("."), Path::new("..")));
    assert!(escapes_repository(
        Path::new("./."),
        Path::new("./../outside")
    ));
    assert!(!escapes_repository(Path::new("."), Path::new("./src")));
    assert!(!escapes_repository(Path::new("."), Path::new(".")));
}

#[test]
fn verbatim_windows_root_names_the_config_file_relatively() {
    // Windows `canonicalize` prefixes `\\?\`. The loader still stores the
    // lexical config path from `--config`.
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.loaded_config_path = crate::config::v2::schema::loaded_path::LoadedConfigPath(Some(
        PathBuf::from(r"C:\repo\fixture\scope-src-parent-parent.yml"),
    ));
    assert_eq!(
        config_file_for_finding(Path::new(r"\\?\C:\repo\fixture"), &config),
        "scope-src-parent-parent.yml"
    );

    config.loaded_config_path = crate::config::v2::schema::loaded_path::LoadedConfigPath(Some(
        PathBuf::from(r"C:\other\config.yml"),
    ));
    assert_eq!(
        config_file_for_finding(Path::new(r"\\?\C:\repo\fixture"), &config),
        "C:/other/config.yml"
    );
}
