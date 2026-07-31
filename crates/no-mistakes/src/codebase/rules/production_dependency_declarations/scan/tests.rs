use super::*;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-dependency-declarations")
            .join(name),
    )
}

fn dummy_manifest() -> PackageManifest {
    PackageManifest::load(
        Path::new("/does/not/exist/package.json"),
        &crate::codebase::rules::source_store_for_files(&[]),
    )
}

fn value_import(specifier: &str) -> FileImport {
    FileImport {
        line: 1,
        specifier: specifier.to_string(),
        kind: ImportKind::Static,
    }
}

#[test]
fn allowed_fields_defaults_when_unconfigured() {
    let fields = allowed_fields(&Options::default()).unwrap();
    assert_eq!(
        fields,
        BTreeSet::from([
            "dependencies".to_string(),
            "optionalDependencies".to_string(),
            "peerDependencies".to_string(),
        ])
    );
}

#[test]
fn allowed_fields_accepts_a_configured_subset() {
    let opts = Options {
        allowed_fields: vec!["dependencies".to_string()],
        ..Options::default()
    };
    assert_eq!(
        allowed_fields(&opts).unwrap(),
        BTreeSet::from(["dependencies".to_string()])
    );
}

#[test]
fn allowed_fields_rejects_an_unsupported_field_name() {
    let opts = Options {
        allowed_fields: vec!["bogusField".to_string()],
        ..Options::default()
    };
    let error = allowed_fields(&opts).unwrap_err();
    assert!(error.contains("unsupported field 'bogusField'"));
}

#[test]
fn test_file_patterns_defaults_when_unconfigured() {
    assert_eq!(
        test_file_patterns(&Options::default()),
        DEFAULT_TEST_FILE_PATTERNS
            .iter()
            .map(|pattern| pattern.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_file_patterns_uses_configured_patterns() {
    let opts = Options {
        test_file_patterns: vec!["**/*.spec.*".to_string()],
        ..Options::default()
    };
    assert_eq!(test_file_patterns(&opts), vec!["**/*.spec.*".to_string()]);
}

#[test]
fn build_globset_rejects_an_invalid_pattern() {
    assert!(build_globset(&["[".to_string()]).is_err());
}

#[test]
fn build_globset_accepts_valid_patterns() {
    let globset = build_globset(&["**/*.test.*".to_string()]).unwrap();
    assert!(globset.is_match("packages/lib/index.test.mts"));
    assert!(!globset.is_match("packages/lib/index.mts"));
}

#[test]
fn run_reports_a_config_finding_for_an_unsupported_allowed_field() {
    let root = fixture_root("dependencies-declared");
    let opts = Options {
        allowed_fields: vec!["bogusField".to_string()],
        ..Options::default()
    };
    let sources = crate::codebase::rules::source_store_for_files(&[]);

    let findings = run(&root, &[], &opts, &[], &sources).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("unsupported field"));
}

#[test]
fn run_reports_a_config_finding_for_an_invalid_test_file_pattern() {
    let root = fixture_root("dependencies-declared");
    let opts = Options {
        test_file_patterns: vec!["[".to_string()],
        ..Options::default()
    };
    let sources = crate::codebase::rules::source_store_for_files(&[]);

    let findings = run(&root, &[], &opts, &[], &sources).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("invalid glob pattern"));
}

#[test]
fn run_returns_no_findings_when_no_packages_are_discovered() {
    let root = fixture_root("dependencies-declared");
    let sources = crate::codebase::rules::source_store_for_files(&[]);

    let findings = run(&root, &[], &Options::default(), &[], &sources).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn emit_finding_ignores_type_only_imports() {
    let mut findings = Vec::new();
    let import = FileImport {
        line: 1,
        specifier: "@acme/tool".to_string(),
        kind: ImportKind::Type,
    };
    emit_finding(
        Path::new("/repo"),
        Path::new("/repo/packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &dummy_manifest(),
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );
    assert!(findings.is_empty());
}

#[test]
fn emit_finding_ignores_relative_specifiers() {
    let mut findings = Vec::new();
    let import = value_import("./sibling");
    emit_finding(
        Path::new("/repo"),
        Path::new("/repo/packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &dummy_manifest(),
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );
    assert!(findings.is_empty());
}

#[test]
fn emit_finding_ignores_hash_prefixed_specifiers() {
    let mut findings = Vec::new();
    let import = value_import("#internal");
    emit_finding(
        Path::new("/repo"),
        Path::new("/repo/packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &dummy_manifest(),
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );
    assert!(findings.is_empty());
}

#[test]
fn emit_finding_ignores_unparseable_specifiers() {
    let mut findings = Vec::new();
    let import = value_import("/abs/path");
    emit_finding(
        Path::new("/repo"),
        Path::new("/repo/packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &dummy_manifest(),
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );
    assert!(findings.is_empty());
}

#[test]
fn emit_finding_ignores_self_reference_imports() {
    let mut findings = Vec::new();
    let import = value_import("@acme/lib");
    emit_finding(
        Path::new("/repo"),
        Path::new("/repo/packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &dummy_manifest(),
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );
    assert!(findings.is_empty());
}

#[test]
fn emit_finding_ignores_node_builtin_imports() {
    let mut findings = Vec::new();
    let import = value_import("node:fs");
    emit_finding(
        Path::new("/repo"),
        Path::new("/repo/packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &dummy_manifest(),
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );
    assert!(findings.is_empty());
}

#[test]
fn emit_finding_pushes_a_dev_only_finding_for_a_dev_dependency_only_package() {
    let root = fixture_root("dev-only-production-import");
    let manifest_path = root.join("packages/lib/package.json");
    let sources =
        crate::codebase::rules::source_store_for_files(std::slice::from_ref(&manifest_path));
    let manifest = PackageManifest::load(&manifest_path, &sources);

    let mut findings = Vec::new();
    let import = value_import("@acme/tool");
    emit_finding(
        &root,
        &root.join("packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &manifest,
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("@acme/tool"));
}

#[test]
fn emit_finding_pushes_an_undeclared_finding_for_an_undeclared_package() {
    let root = fixture_root("undeclared-import");
    let manifest_path = root.join("packages/lib/package.json");
    let sources =
        crate::codebase::rules::source_store_for_files(std::slice::from_ref(&manifest_path));
    let manifest = PackageManifest::load(&manifest_path, &sources);

    let mut findings = Vec::new();
    let import = value_import("left-pad");
    emit_finding(
        &root,
        &root.join("packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &manifest,
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("left-pad"));
}

#[test]
fn emit_finding_is_silent_for_a_declared_dependency() {
    let root = fixture_root("dependencies-declared");
    let manifest_path = root.join("packages/lib/package.json");
    let sources =
        crate::codebase::rules::source_store_for_files(std::slice::from_ref(&manifest_path));
    let manifest = PackageManifest::load(&manifest_path, &sources);

    let mut findings = Vec::new();
    let import = value_import("@acme/tool");
    emit_finding(
        &root,
        &root.join("packages/lib/index.mts"),
        &import,
        "@acme/lib",
        &manifest,
        &BTreeSet::from(["dependencies".to_string()]),
        &mut findings,
    );

    assert!(findings.is_empty());
}
