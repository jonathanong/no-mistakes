use super::super::{
    changed_unmodeled_installation_sections, impact_importer_paths, impact_names, parse,
    parse_importers, validate_for_planning, PnpmValidationError,
};
use crate::codebase::lockfile::{diff, ResolutionKind};
use crate::codebase::pnpm_lock::parse_pnpm_lock;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn issue_lock(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/pnpm12-issue-1035")
        .join(name)
        .join("pnpm-lock.yaml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn v12(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lockfile/pnpm12")
        .join(name)
        .join("pnpm-lock.yaml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn package_names(content: &str) -> BTreeSet<String> {
    parse(content)
        .into_iter()
        .map(|package| package.name)
        .collect()
}

fn changed(old: &str, new: &str) -> Vec<String> {
    diff(&parse(old), &parse(new)).changed
}

fn added(old: &str, new: &str) -> Vec<String> {
    diff(&parse(old), &parse(new)).added
}

#[test]
fn single_document_project_keeps_v9_shape() {
    let content = v12("single-doc-project");
    assert!(validate_for_planning(&content).is_ok());
    let importers = parse_importers(&content);
    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].path, ".");
    assert_eq!(importers[0].dependencies[0].alias, "react");
    assert_eq!(
        package_names(&content),
        BTreeSet::from(["react".to_string()])
    );
}

#[test]
fn two_document_lockfile_unions_packages_and_reads_project_importers() {
    // The env document is a valid lockfile whose `.` importer has no project
    // dependencies. The project graph is the last document, and it is not merged
    // into the env document.
    let content = v12("two-doc-package-manager");
    assert!(validate_for_planning(&content).is_ok());
    assert_eq!(
        package_names(&content),
        BTreeSet::from(["pnpm".to_string(), "react".to_string()])
    );
    let importers = parse_importers(&content);
    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].dependencies.len(), 1);
    assert_eq!(importers[0].dependencies[0].alias, "react");
    assert_eq!(importers[0].dependencies[0].version, "19.0.0");
}

#[test]
fn leading_document_marker_does_not_invent_an_empty_document() {
    let content = v12("two-doc-leading-marker");
    // A retained null document from the leading `---` would fail validation.
    assert!(validate_for_planning(&content).is_ok());
    assert!(package_names(&content).contains("react"));
}

#[test]
fn separator_without_leading_marker_still_uses_the_last_document() {
    let content = v12("two-doc-separator-without-leading-marker");
    assert!(validate_for_planning(&content).is_ok());
    assert_eq!(parse_importers(&content)[0].dependencies[0].alias, "react");
}

#[test]
fn project_package_change_ignores_the_unchanged_env_document() {
    let old = v12("project-change-old");
    let new = v12("project-change-new");
    assert_eq!(changed(&old, &new), vec!["react".to_string()]);
    assert!(added(&old, &new).is_empty());
    assert!(changed_unmodeled_installation_sections(&old, &new).is_empty());
    let names = impact_names(&old, &new, ["react".to_string()]);
    assert!(names.contains(&"react".to_string()), "{names:?}");
    assert!(!names.contains(&"pnpm".to_string()), "{names:?}");
    assert_eq!(
        impact_importer_paths(&old, &new, &names).get("react"),
        Some(&vec!["packages/app".to_string()])
    );
}

#[test]
fn env_package_change_is_a_package_diff_without_an_installation_warning() {
    let old = v12("env-change-old");
    let new = v12("env-change-new");
    assert_eq!(changed(&old, &new), vec!["pnpm".to_string()]);
    assert!(changed_unmodeled_installation_sections(&old, &new).is_empty());
    let names = impact_names(&old, &new, changed(&old, &new));
    assert!(names.contains(&"pnpm".to_string()), "{names:?}");
    assert!(
        !impact_importer_paths(&old, &new, &names).contains_key("pnpm"),
        "the env importer is not a workspace importer"
    );
    assert!(impact_names(&old, &new, std::iter::empty()).is_empty());
}

#[test]
fn gaining_an_env_document_does_not_drop_project_settings() {
    let old = v12("gained-env-document-old");
    let new = v12("gained-env-document-new");
    assert_eq!(added(&old, &new), vec!["pnpm".to_string()]);
    assert!(changed(&old, &new).is_empty());
    assert!(changed_unmodeled_installation_sections(&old, &new).is_empty());
    assert!(package_names(&new).contains("react"));
}

#[test]
fn config_dependency_git_package_is_in_the_union_inventory() {
    let content = v12("config-dep-git");
    let packages = parse(&content);
    let plugin = packages
        .iter()
        .find(|package| package.name == "github.com/org/my-plugin")
        .unwrap();
    assert_eq!(plugin.kind, ResolutionKind::Git);
    assert_eq!(plugin.fingerprint, "abc123");
    assert!(package_names(&content).contains("left-pad"));
    assert!(package_names(&content).contains("react"));
    let importers = parse_importers(&content);
    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].dependencies[0].alias, "react");
}

#[test]
fn config_dependency_tarball_is_in_the_union_inventory() {
    let content = v12("config-dep-tarball");
    let plugin = parse(&content)
        .into_iter()
        .find(|package| package.name == "config-plugin")
        .unwrap();
    assert_eq!(plugin.kind, ResolutionKind::Tarball);
    assert_eq!(plugin.fingerprint, "sha512-tb");
    assert!(package_names(&content).contains("react"));
}

#[test]
fn workspace_importers_come_from_the_project_document() {
    let importers = parse_importers(&v12("workspace-importers"));
    assert_eq!(
        importers
            .iter()
            .map(|importer| importer.path.as_str())
            .collect::<Vec<_>>(),
        vec![".", "packages/app", "packages/lib"]
    );
    assert_eq!(importers[0].dependencies[0].alias, "react");
    assert_eq!(importers[1].dependencies[0].alias, "lodash");
    assert_eq!(importers[2].dependencies[0].alias, "chalk");
}

#[test]
fn snapshot_leaf_change_stays_inside_the_project_graph() {
    let old = v12("snapshot-leaf-old");
    let new = v12("snapshot-leaf-new");
    let names = impact_names(&old, &new, std::iter::empty());
    assert_eq!(
        names,
        vec!["leaf".to_string(), "middle".to_string()],
        "{names:?}"
    );
    assert_eq!(
        impact_importer_paths(&old, &new, &names).get("middle"),
        Some(&vec!["app-v1".to_string()])
    );
}

#[test]
fn project_aliases_survive_a_preceding_env_document() {
    let importers = parse_importers(&v12("alias-importers"));
    assert_eq!(importers.len(), 1);
    let deps = &importers[0].dependencies;
    assert_eq!(deps[0].alias, "domain-alias");
    assert_eq!(deps[0].resolution_name.as_deref(), Some("@acme/domain"));
    assert_eq!(deps[1].alias, "secret-alias");
    assert_eq!(deps[1].resolution_name.as_deref(), Some("@acme/secret"));
}

#[test]
fn version_only_peer_suffix_walks_project_importers() {
    let old = v12("scoped-peers-old");
    let new = v12("scoped-peers-new");
    let names = impact_names(&old, &new, std::iter::empty());
    let paths = impact_importer_paths(&old, &new, &names);
    assert_eq!(names, vec!["middle".to_string()], "{names:?}");
    assert_eq!(
        paths.get("middle"),
        Some(&vec!["react-app".to_string(), "vue-app".to_string()])
    );
}

#[test]
fn cut_peer_cycle_keeps_the_remaining_edge() {
    let old = v12("peer-cycle-cut-old");
    let new = v12("peer-cycle-cut-new");
    let names = impact_names(&old, &new, std::iter::empty());
    let paths = impact_importer_paths(&old, &new, &names);
    assert!(names.contains(&"right".to_string()), "{names:?}");
    assert!(!names.contains(&"tooling".to_string()), "{names:?}");
    assert_eq!(paths.get("right"), Some(&vec!["packages/app".to_string()]));
}

#[test]
fn revision_keeps_integrity_as_the_registry_fingerprint() {
    let package = parse(&v12("revision-with-integrity"))
        .into_iter()
        .find(|package| package.name == "lodash")
        .unwrap();
    assert_eq!(package.kind, ResolutionKind::Registry);
    assert_eq!(package.fingerprint, "sha512-replacement");
    assert_eq!(package.version, "4.17.21");
}

#[test]
fn revision_alone_does_not_change_package_identity() {
    let old = v12("revision-same-bytes-old");
    let new = v12("revision-same-bytes-new");
    assert!(diff(&parse(&old), &parse(&new)).is_empty());
}

#[test]
fn replacement_digest_is_a_package_change() {
    assert_eq!(
        changed(
            &v12("revision-replacement-old"),
            &v12("revision-replacement-new")
        ),
        vec!["lodash".to_string()]
    );
}

#[test]
fn project_resolution_kinds_are_not_hidden_by_the_env_document() {
    let content = v12("git-tarball-directory");
    let mut kinds = std::collections::BTreeMap::new();
    for package in parse_pnpm_lock(&content) {
        kinds.insert(package.key, package.resolution_kind);
    }
    assert_eq!(
        kinds.get("pnpm@12.3.4").map(String::as_str),
        Some("integrity")
    );
    assert_eq!(
        kinds.get("github.com/org/repo").map(String::as_str),
        Some("repo")
    );
    assert_eq!(
        kinds.get("some-tarball@1.0.0").map(String::as_str),
        Some("tarball")
    );
    assert_eq!(
        kinds.get("local-pkg@1.0.0").map(String::as_str),
        Some("directory")
    );
    assert_eq!(
        kinds.get("commit-only@1.0.0").map(String::as_str),
        Some("commit")
    );
}

#[test]
fn project_settings_change_is_still_unmodeled() {
    assert_eq!(
        changed_unmodeled_installation_sections(
            &v12("settings-change-old"),
            &v12("settings-change-new")
        ),
        vec!["settings".to_string()]
    );
}

#[test]
fn project_overrides_and_patches_are_not_hidden_by_the_env_document() {
    assert_eq!(
        changed_unmodeled_installation_sections(
            &v12("overrides-patched-old"),
            &v12("overrides-patched-new")
        ),
        vec!["overrides".to_string(), "patchedDependencies".to_string()]
    );
}

#[test]
fn project_catalog_change_is_unmodeled_and_packages_still_parse() {
    let old = v12("catalogs-change-old");
    let new = v12("catalogs-change-new");
    assert_eq!(
        changed_unmodeled_installation_sections(&old, &new),
        vec!["catalogs".to_string()]
    );
    assert!(package_names(&new).contains("react"));
}

#[test]
fn broken_project_document_fails_the_whole_file() {
    let content = v12("malformed-project-document");
    assert!(matches!(
        validate_for_planning(&content),
        Err(PnpmValidationError::Malformed)
    ));
    assert!(parse(&content).is_empty());
}

#[test]
fn broken_env_document_fails_the_whole_file() {
    let content = v12("malformed-env-document");
    assert!(matches!(
        validate_for_planning(&content),
        Err(PnpmValidationError::Malformed)
    ));
    assert!(parse(&content).is_empty());
}

#[test]
fn unsupported_and_non_mapping_project_documents_are_rejected() {
    for name in [
        "unsupported-major",
        "non-mapping-packages",
        "missing-sections",
        "bool-version",
    ] {
        assert!(
            matches!(
                validate_for_planning(&v12(name)),
                Err(PnpmValidationError::UnsupportedSchema)
            ),
            "{name}"
        );
    }
}

#[test]
fn issue_1035_root_importer_is_the_project_document() {
    // Issue #1035: `.` in the env document pins pnpm. The project document's
    // `.` importer is empty and must win.
    let content = issue_lock("two-doc");
    assert!(validate_for_planning(&content).is_ok());
    let root = parse_importers(&content)
        .into_iter()
        .find(|importer| importer.path == ".")
        .unwrap();
    assert!(root.dependencies.is_empty(), "{root:?}");
    let names = package_names(&content);
    assert!(names.contains("kind-of"));
    assert!(names.contains("@pnpm/exe.darwin-arm64"));
    assert!(names.contains("pnpm"));
}

#[test]
fn issue_1035_second_document_diff_matches_dropping_is_number_7() {
    // A pnpm pin lives only in the env document. Bumping is-number in the
    // project document is the same package change the single-document form has.
    let two = diff(
        &parse(&issue_lock("two-doc")),
        &parse(&issue_lock("diff-new")),
    );
    assert_eq!(two.changed, vec!["is-number".to_string()]);
    assert!(two.added.is_empty(), "{two:?}");
    assert!(two.removed.is_empty(), "{two:?}");
    let single = diff(
        &parse(&issue_lock("single-doc")),
        &parse(&issue_lock("single-doc-new")),
    );
    assert_eq!(single.changed, two.changed);
}

#[test]
fn non_env_prefix_is_an_explicit_error() {
    assert!(matches!(
        validate_for_planning(&issue_lock("non-env")),
        Err(PnpmValidationError::NotEnvPrefix)
    ));
}

#[test]
fn schema_majors_five_through_nine_stay_supported() {
    for name in ["v5", "v6", "v9", "two-doc-package-manager"] {
        assert!(validate_for_planning(&v12(name)).is_ok(), "{name}");
    }
    let v5 = parse(&v12("v5"));
    assert_eq!(v5[0].name, "acme-sample");
    assert_eq!(v5[0].version, "4.17.21");
    let v6 = parse(&v12("v6"));
    assert_eq!(v6[0].name, "acme-sample");
    assert_eq!(v6[0].version, "4.17.21");
}
