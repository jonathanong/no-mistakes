use super::{resolve, resolve_with};

fn error_contains(yaml: &str, files: &[(&str, &str)], expected: &str) -> bool {
    resolve(yaml, files)
        .load_errors
        .iter()
        .any(|error| error.message.contains(expected))
}

#[test]
fn resolve_closure_rejects_non_local_and_escaping_includes() {
    let cases = [
        (
            "include:\n  ./wild*.yaml: [packages]\n",
            "is not a local YAML file",
        ),
        (
            "include:\n  ../outside.yaml: [packages]\n",
            "escapes the workspace root",
        ),
    ];
    for (yaml, expected) in cases {
        assert!(error_contains(yaml, &[], expected), "{expected}");
    }
}

#[test]
fn resolve_closure_rejects_a_non_mapping_root() {
    let Err(error) = resolve_with("- item\n", &[]) else {
        panic!("a sequence root is not a pnpm workspace mapping");
    };
    assert!(error.to_string().contains("must be a mapping"));
}

#[test]
fn resolve_closure_catalog_and_package_edges() {
    let phantom = resolve(
        "catalogs:\n  react:\n    react: 18.0.0\ninclude:\n  ./missing.yaml: [catalogs]\n",
        &[],
    );
    assert!(phantom
        .duplicates
        .iter()
        .any(|setting| setting == "catalogs"));

    let odd_key = resolve("catalogs:\n  ? [1]\n  : {}\n", &[]);
    assert!(odd_key.catalogs.named.is_empty());

    let slots = resolve(
        "include:\n  ./pnpm/slots.yaml: [catalogs., catalogs.default, catalogs.a.b, catalogs.react]\n",
        &[("/pnpm/slots.yaml", "catalogs:\n  react:\n    react: 18.0.0\n")],
    );
    assert_eq!(
        slots
            .catalogs
            .get("react", "react")
            .map(|hit| hit.specifier.as_str()),
        Some("18.0.0")
    );
    assert!(!slots.catalogs.named.contains_key("default"));
    assert!(!slots.catalogs.named.contains_key("a.b"));

    let missing_slot = resolve(
        "include:\n  ./pnpm/claim.yaml: [catalogs.react]\n",
        &[("/pnpm/claim.yaml", "packages: []\n")],
    );
    assert!(missing_slot.catalogs.named.contains_key("react"));
    assert!(missing_slot.catalogs.get("react", "react").is_none());

    let not_mapping = resolve("catalog: nope\n", &[]);
    assert!(not_mapping.catalogs.default_entries.is_empty());

    let mixed = resolve("catalog:\n  foo: 1\n  bar: \"1.0.0\"\n", &[]);
    assert!(mixed.catalogs.get("", "foo").is_none());
    assert_eq!(
        mixed
            .catalogs
            .get("", "bar")
            .map(|hit| hit.specifier.as_str()),
        Some("1.0.0")
    );

    let bad_list = resolve("packages: [1]\n", &[]);
    assert!(bad_list.packages_invalid);
    assert!(
        matches!(bad_list.packages, super::super::super::PackageGlobs::List(ref list) if list.is_empty())
    );

    let null_packages = resolve(
        "include:\n  ./pnpm/packages.yaml: [packages]\n",
        &[("/pnpm/packages.yaml", "packages: null\n")],
    );
    assert!(!null_packages.packages_invalid);
    assert!(super::list(&null_packages).is_empty());

    let rooted = resolve(
        "include:\n  ./packages.yaml: [packages]\n",
        &[("/packages.yaml", "packages:\n  - apps/*\n")],
    );
    assert_eq!(super::list(&rooted), ["apps/*"]);
}
