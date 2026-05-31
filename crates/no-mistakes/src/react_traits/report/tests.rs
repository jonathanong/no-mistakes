use super::types::{FileConfig, RootConfig};

#[test]
fn into_file_config_no_react_traits_section() {
    let root = RootConfig::default();
    let fc = root.into_file_config();
    assert!(fc.frontend_root.is_none());
    assert!(fc.assert_no_fetch.is_none());
}

#[test]
fn into_file_config_top_level_fields() {
    let root = RootConfig {
        frontend_root: Some("components".into()),
        assert_no_fetch: Some(true),
        react_traits: None,
    };
    let fc = root.into_file_config();
    assert_eq!(fc.frontend_root.as_deref(), Some("components"));
    assert_eq!(fc.assert_no_fetch, Some(true));
}

#[test]
fn into_file_config_react_traits_section_assert_no_fetch() {
    let root = RootConfig {
        react_traits: Some(FileConfig {
            frontend_root: None,
            assert_no_fetch: Some(true),
        }),
        ..Default::default()
    };
    let fc = root.into_file_config();
    assert!(fc.frontend_root.is_none());
    assert_eq!(fc.assert_no_fetch, Some(true));
}

#[test]
fn into_file_config_react_traits_section_frontend_root() {
    let root = RootConfig {
        react_traits: Some(FileConfig {
            frontend_root: Some("src/app".into()),
            assert_no_fetch: None,
        }),
        ..Default::default()
    };
    let fc = root.into_file_config();
    assert_eq!(fc.frontend_root.as_deref(), Some("src/app"));
    assert!(fc.assert_no_fetch.is_none());
}

#[test]
fn into_file_config_full_override() {
    let root = RootConfig {
        react_traits: Some(FileConfig {
            frontend_root: Some("src/app".into()),
            assert_no_fetch: Some(true),
        }),
        ..Default::default()
    };
    let fc = root.into_file_config();
    assert_eq!(fc.frontend_root.as_deref(), Some("src/app"));
    assert_eq!(fc.assert_no_fetch, Some(true));
}

#[test]
fn into_file_config_react_traits_section_overrides_top_level_fields() {
    let root = RootConfig {
        frontend_root: Some("components".into()),
        assert_no_fetch: Some(false),
        react_traits: Some(FileConfig {
            frontend_root: Some("src/app".into()),
            assert_no_fetch: Some(true),
        }),
    };
    let fc = root.into_file_config();
    assert_eq!(fc.frontend_root.as_deref(), Some("src/app"));
    assert_eq!(fc.assert_no_fetch, Some(true));
}
