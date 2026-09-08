use super::*;

#[test]
fn extends_specs_accept_local_and_package_specs() {
    let value: Value =
        serde_yaml::from_str("extends: [./base.yml, ../parent.yml, package-config]").unwrap();
    assert_eq!(
        extends_specs(&value, "extends").unwrap(),
        vec!["./base.yml", "../parent.yml", "package-config"]
    );
}

#[test]
fn extends_specs_reject_invalid_shapes_and_portable_absolute_forms() {
    for source in [
        "extends: true",
        "extends: [./base.yml, false]",
        "extends: ''",
        "extends: C:base.yml",
        "extends: '\\\\server\\share'",
        "extends: /absolute.yml",
    ] {
        let value: Value = serde_yaml::from_str(source).unwrap();
        assert!(extends_specs(&value, "extends").is_err(), "{source}");
    }
    assert!(extends_specs(&Value::Null, "extends").unwrap().is_empty());
}
