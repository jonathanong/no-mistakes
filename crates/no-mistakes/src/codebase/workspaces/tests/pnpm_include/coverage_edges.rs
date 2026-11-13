use std::path::PathBuf;

fn saved(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/coverage-region-edges/pnpm")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn resolve_saved(root: &str, files: &[(&str, &str)]) -> super::PnpmInclude {
    let loaded: Vec<(String, String)> = files
        .iter()
        .map(|(suffix, file)| ((*suffix).to_string(), saved(file)))
        .collect();
    let pairs: Vec<(&str, &str)> = loaded
        .iter()
        .map(|(suffix, body)| (suffix.as_str(), body.as_str()))
        .collect();
    super::resolve(&saved(root), &pairs)
}

fn error_contains(root: &str, expected: &str) -> bool {
    super::resolve(&saved(root), &[])
        .load_errors
        .iter()
        .any(|error| error.message.contains(expected))
}

#[test]
fn resolve_closure_rejects_non_local_and_escaping_includes() {
    let cases = [
        ("non-local.yaml", "is not a local YAML file"),
        ("escape.yaml", "escapes the workspace root"),
    ];
    for (root, expected) in cases {
        assert!(error_contains(root, expected), "{expected}");
    }
}

#[test]
fn resolve_closure_rejects_a_non_mapping_root() {
    let Err(error) = super::resolve_with(&saved("sequence.yaml"), &[]) else {
        panic!("a sequence root is not a pnpm workspace mapping");
    };
    assert!(error.to_string().contains("must be a mapping"));
}

#[test]
fn resolve_closure_catalog_and_package_edges() {
    let phantom = resolve_saved("phantom.yaml", &[]);
    assert!(phantom
        .duplicates
        .iter()
        .any(|setting| setting == "catalogs"));

    let odd_key = resolve_saved("odd-key.yaml", &[]);
    assert!(odd_key.catalogs.named.is_empty());

    let slots = resolve_saved(
        "slots-root.yaml",
        &[("/pnpm/slots.yaml", "slots-body.yaml")],
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

    let missing_slot = resolve_saved(
        "claim-root.yaml",
        &[("/pnpm/claim.yaml", "claim-body.yaml")],
    );
    assert!(missing_slot.catalogs.named.contains_key("react"));
    assert!(missing_slot.catalogs.get("react", "react").is_none());

    let not_mapping = resolve_saved("catalog-nope.yaml", &[]);
    assert!(not_mapping.catalogs.default_entries.is_empty());

    let mixed = resolve_saved("catalog-mixed.yaml", &[]);
    assert!(mixed.catalogs.get("", "foo").is_none());
    assert_eq!(
        mixed
            .catalogs
            .get("", "bar")
            .map(|hit| hit.specifier.as_str()),
        Some("1.0.0")
    );

    let bad_list = resolve_saved("packages-number.yaml", &[]);
    assert!(bad_list.packages_invalid);
    assert!(
        matches!(bad_list.packages, super::super::super::PackageGlobs::List(ref list) if list.is_empty())
    );

    let null_packages = resolve_saved(
        "null-root.yaml",
        &[("/pnpm/packages.yaml", "null-body.yaml")],
    );
    assert!(!null_packages.packages_invalid);
    assert!(super::list(&null_packages).is_empty());

    let rooted = resolve_saved(
        "rooted-root.yaml",
        &[("/packages.yaml", "rooted-body.yaml")],
    );
    assert_eq!(super::list(&rooted), ["apps/*"]);
}
