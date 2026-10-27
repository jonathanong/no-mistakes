use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn cfg(name: &str) -> PathBuf {
    rules_path("forbidden-workspace-closure", "configs", name)
}

fn root(name: &str) -> PathBuf {
    rules_path("forbidden-workspace-closure", name, ".")
}

fn fixtures_root(tree: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules")
            .join(tree)
            .join(name),
    )
}

#[test]
fn forbidden_workspace_closure_json_matches_snapshot() {
    let secret = cfg("acme-app-secret.yml");
    let packages = cfg("acme-app-secret-include-packages.yml");
    let acme_root = cfg("acme-root-secret.yml");
    assert_saved_fixture(
        "forbidden_workspace_closure",
        &[
            ("direct_external", root("direct-external"), secret.clone()),
            ("transitive", root("transitive-workspace"), secret.clone()),
            ("outside_closure", root("pass"), secret.clone()),
            (
                "dependency_types_default",
                root("dependency-types"),
                secret.clone(),
            ),
            (
                "dependency_types_dev",
                root("dependency-types"),
                cfg("acme-app-secret-dev.yml"),
            ),
            ("glob", root("glob"), cfg("acme-app-secret-glob.yml")),
            ("cycle", root("cycle"), secret.clone()),
            ("diamond", root("diamond"), secret.clone()),
            (
                "manifest_alias",
                root("manifest-alias-workspace"),
                secret.clone(),
            ),
            (
                "exact_workspace_range",
                root("manifest-exact-workspace-range"),
                secret.clone(),
            ),
            (
                "manifest_path_alias",
                root("manifest-path-alias"),
                secret.clone(),
            ),
            (
                "include_start_package",
                root("transitive-workspace"),
                cfg("acme-app-secret-include-app.yml"),
            ),
            (
                "exclude_declaring_manifest",
                root("direct-external"),
                cfg("acme-app-secret-exclude-app.yml"),
            ),
            (
                "catalog_workspace",
                root("catalog-workspace"),
                secret.clone(),
            ),
            (
                "catalog_link_alias",
                root("catalog-link-alias"),
                secret.clone(),
            ),
            ("catalog_semver", root("catalog-semver"), secret.clone()),
            ("catalog_included", root("catalog-included"), secret.clone()),
            (
                "catalog_duplicate",
                root("catalog-duplicate"),
                secret.clone(),
            ),
            (
                "scope_duplicate_catalog",
                root("catalog-duplicate"),
                packages.clone(),
            ),
            (
                "scope_catalog_workspace",
                root("catalog-workspace"),
                packages,
            ),
            (
                "pnpm_root_package",
                root("pnpm-root-package"),
                acme_root.clone(),
            ),
            (
                "npm_alias",
                root("manifest-npm-alias-registry"),
                secret.clone(),
            ),
            (
                "registry_same_name",
                root("manifest-registry-same-name"),
                secret.clone(),
            ),
            (
                "plain_workspace_range",
                root("manifest-plain-workspace-range"),
                secret.clone(),
            ),
            (
                "digit_npm_alias",
                root("manifest-digit-npm-alias"),
                cfg("acme-app-7zip.yml"),
            ),
            (
                "digit_workspace_alias",
                root("manifest-digit-workspace-alias"),
                secret.clone(),
            ),
            (
                "prerelease_workspace_range",
                root("manifest-prerelease-workspace-range"),
                secret.clone(),
            ),
            (
                "x_workspace_range",
                root("manifest-x-workspace-range"),
                secret.clone(),
            ),
            (
                "file_path_alias",
                root("manifest-file-path-alias"),
                secret.clone(),
            ),
            (
                "root_file_path_alias",
                root("manifest-root-file-path-alias"),
                acme_root,
            ),
            ("wildcard_range", root("manifest-wildcard-range"), secret),
        ],
    );
}

#[test]
fn forbidden_workspace_closure_lockfile_json_matches_snapshot() {
    let locked = cfg("acme-app-secret-lock.yml");
    let odd = cfg("fixture-api-is-odd.yml");
    let pnpm12 = fixtures_root("forbidden-workspace-closure", "pnpm12-two-doc");
    let issue = |name: &str| fixtures_root("pnpm12-issue-1035", name);
    assert_saved_fixture(
        "forbidden_workspace_closure_lockfile",
        &[
            ("lockfile_alias", root("lockfile-alias"), locked.clone()),
            (
                "scalar_alias",
                root("lockfile-scalar-alias"),
                locked.clone(),
            ),
            (
                "workspace_alias",
                root("lockfile-workspace-alias"),
                locked.clone(),
            ),
            (
                "exact_range",
                root("lockfile-exact-workspace-range"),
                locked.clone(),
            ),
            (
                "prerelease_range",
                root("lockfile-prerelease-workspace-range"),
                locked.clone(),
            ),
            (
                "x_range",
                root("lockfile-x-workspace-range"),
                locked.clone(),
            ),
            (
                "digit_direct",
                root("lockfile-digit-alias"),
                cfg("acme-app-7zip-lock.yml"),
            ),
            (
                "digit_closure",
                root("lockfile-digit-alias"),
                locked.clone(),
            ),
            ("path_alias", root("lockfile-path-alias"), locked.clone()),
            (
                "registry_same_name",
                root("lockfile-registry-same-name"),
                locked.clone(),
            ),
            (
                "partial_workspace",
                root("lockfile-partial-workspace"),
                locked.clone(),
            ),
            (
                "dependency_type_fields",
                root("lockfile-partial-workspace"),
                cfg("acme-app-secret-lock-fields.yml"),
            ),
            (
                "nested_lockfile",
                root("nested-lockfile"),
                cfg("acme-app-secret-nested-lock.yml"),
            ),
            (
                "root_importer",
                root("root-lockfile"),
                cfg("acme-root-secret-lock.yml"),
            ),
            (
                "optional_dependencies",
                root("lockfile-dependency-types"),
                cfg("acme-app-secret-lock-optional.yml"),
            ),
            (
                "peer_dependencies",
                root("lockfile-dependency-types"),
                cfg("acme-app-secret-lock-peer.yml"),
            ),
            (
                "peer_not_dependency",
                root("lockfile-dependency-types"),
                cfg("acme-app-secret-lock-deps.yml"),
            ),
            (
                "root_path_alias",
                root("lockfile-root-path-alias"),
                cfg("acme-app-secret-root-path-lock.yml"),
            ),
            (
                "root_link_alias",
                root("lockfile-root-link-alias"),
                cfg("acme-app-secret-root-link-lock.yml"),
            ),
            ("pnpm12_project", pnpm12, locked),
            ("issue_1035_is_odd", issue("two-doc"), odd.clone()),
            (
                "issue_1035_is_even",
                issue("two-doc"),
                cfg("fixture-api-is-even.yml"),
            ),
            ("issue_1035_single_doc", issue("single-doc"), odd.clone()),
            // Lockfile parse failures are anchored on the config file. That
            // file has to sit inside the fixture; an external --config path is
            // absolute and is not stable across machines.
            (
                "issue_1035_malformed",
                issue("malformed"),
                issue("malformed").join("snapshot-config.yml"),
            ),
            (
                "issue_1035_non_env",
                issue("non-env"),
                issue("non-env").join("snapshot-config.yml"),
            ),
            (
                "pnpm12_env_only",
                fixtures_root("forbidden-workspace-closure", "pnpm12-env-only-forbidden"),
                fixtures_root("forbidden-workspace-closure", "pnpm12-env-only-forbidden")
                    .join("snapshot-config.yml"),
            ),
        ],
    );
}
