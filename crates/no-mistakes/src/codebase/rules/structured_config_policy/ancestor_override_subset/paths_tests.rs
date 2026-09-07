use super::resolution::paths::local_specifier;

#[test]
fn classifies_posix_windows_and_package_extends() {
    assert_eq!(
        local_specifier("./base.json").unwrap(),
        Some("./base.json".to_string())
    );
    assert_eq!(
        local_specifier("..\\base.json").unwrap(),
        Some("../base.json".to_string())
    );
    assert_eq!(local_specifier("@scope/config").unwrap(), None);
    for path in [
        "",
        "  ",
        "/tmp/base.json",
        "C:\\outside.json",
        "C:base.json",
        "\\\\server\\share\\base.json",
    ] {
        assert!(local_specifier(path).is_err(), "{path:?}");
    }
}
