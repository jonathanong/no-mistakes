use super::{
    diff, fixture, malformed::assert_rejected_fixtures, KeyChanges, KeyFormat, KeyOptions,
};

#[test]
fn keyed_npm_v1_compares_empty_and_nested_dependencies() {
    let options = KeyOptions {
        format: KeyFormat::Npm,
        array: None,
        id_field: None,
    };
    let base = fixture("npm", "v1-empty", "json");
    let head = fixture("npm", "v1-nested", "json");
    assert_eq!(
        diff(&options, Some(&base), Some(&head)).unwrap(),
        KeyChanges {
            added: vec!["child".into(), "parent".into()],
            ..KeyChanges::default()
        }
    );
}

#[test]
fn keyed_npm_rejects_malformed_lockfiles_across_versions() {
    let options = KeyOptions {
        format: KeyFormat::Npm,
        array: None,
        id_field: None,
    };
    assert_rejected_fixtures(
        &options,
        "npm",
        "json",
        &[
            ("invalid-json", "invalid npm lockfile JSON"),
            ("unsupported-version", "unsupported npm lockfile version"),
            ("v1-invalid-dependencies", "invalid npm dependencies"),
            (
                "v1-invalid-nested-dependencies",
                "invalid nested npm dependencies",
            ),
            (
                "v1-invalid-nested-record",
                "invalid npm package record \"child\"",
            ),
            (
                "invalid-package-record",
                "invalid npm package record \"node_modules/alpha\"",
            ),
        ],
    );
}

#[test]
fn keyed_swift_rejects_unsupported_resolutions_and_ambiguous_pins() {
    let options = KeyOptions {
        format: KeyFormat::Swift,
        array: None,
        id_field: None,
    };
    assert_rejected_fixtures(
        &options,
        "swift",
        "json",
        &[
            (
                "missing-version",
                "invalid Swift lockfile: UnsupportedSchema",
            ),
            (
                "unsupported-version",
                "invalid Swift lockfile: UnsupportedSchema",
            ),
            ("missing-identity", "invalid Swift lockfile: Malformed"),
            ("conflicting-pins", "invalid Swift lockfile: Malformed"),
            ("invalid-json", "invalid Swift lockfile: Malformed"),
        ],
    );
}

#[test]
fn keyed_nuget_rejects_malformed_framework_and_package_entries() {
    let options = KeyOptions {
        format: KeyFormat::Nuget,
        array: None,
        id_field: None,
    };
    let invalid_schema = "unsupported NuGet lockfile schema";
    assert_rejected_fixtures(
        &options,
        "nuget",
        "json",
        &[
            ("missing-dependencies", invalid_schema),
            ("invalid-framework", invalid_schema),
            ("invalid-package-record", invalid_schema),
            ("invalid-json", "invalid NuGet lockfile JSON"),
        ],
    );
}

#[test]
fn keyed_pnpm_rejects_nonmapping_package_records() {
    let options = KeyOptions {
        format: KeyFormat::Pnpm,
        array: None,
        id_field: None,
    };
    assert_rejected_fixtures(
        &options,
        "pnpm",
        "yaml",
        &[(
            "invalid-package-record",
            "invalid pnpm lockfile: UnsupportedSchema",
        )],
    );
}
