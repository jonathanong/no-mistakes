use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "package-json-registry-only";

fn fixture(name: &str) -> PathBuf {
    rules_path(RULE, "fixture", name)
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

fn saved_root(base: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules")
            .join(base)
            .join(name),
    )
}

#[test]
fn package_json_registry_only_json_matches_snapshot() {
    let empty = config("empty.yml");
    let lockfile = config("lockfile.yml");
    assert_saved_fixture(
        "package_json_registry_only",
        &[
            ("all_dep_fields", fixture("all-dep-fields"), empty.clone()),
            ("finding_message", fixture("finding-message"), empty.clone()),
            ("pass", fixture("pass"), empty.clone()),
            ("fail", fixture("fail"), empty),
            (
                "lockfile_tarball",
                fixture("lockfile-tarball"),
                lockfile.clone(),
            ),
            (
                "lockfile_excluded",
                fixture("lockfile-tarball"),
                config("lockfile-exclude.yml"),
            ),
            (
                "lockfile_integrity",
                fixture("lockfile-integrity"),
                lockfile.clone(),
            ),
            (
                "lockfile_invalid_yaml",
                fixture("lockfile-invalid-yaml"),
                lockfile.clone(),
            ),
            (
                "lockfile_no_packages",
                fixture("lockfile-no-packages"),
                lockfile.clone(),
            ),
            (
                "lockfile_no_resolution",
                fixture("lockfile-no-resolution"),
                lockfile.clone(),
            ),
            ("scopes_a", fixture("scopes-filter"), config("scopes-a.yml")),
            (
                "missing_lockfile",
                fixture("missing-input"),
                config("missing-lockfile.yml"),
            ),
            (
                "broken_json",
                fixture("unanalyzable"),
                config("broken-include.yml"),
            ),
            (
                "both_projects_report",
                fixture("project-lockfile-scope"),
                config("both-report.yml"),
            ),
            (
                "both_projects_advise",
                fixture("project-lockfile-scope"),
                config("both-advise.yml"),
            ),
            (
                "both_projects_ignore",
                fixture("project-lockfile-scope"),
                config("both-ignore.yml"),
            ),
            (
                "web_only_report",
                fixture("project-lockfile-scope"),
                config("web-report.yml"),
            ),
            (
                "web_only_advise",
                fixture("project-lockfile-scope"),
                config("web-advise.yml"),
            ),
            (
                "web_only_ignore",
                fixture("project-lockfile-scope"),
                config("web-ignore.yml"),
            ),
            (
                "excluded_admin_report",
                fixture("project-lockfile-scope"),
                config("excluded-report.yml"),
            ),
            (
                "excluded_admin_advise",
                fixture("project-lockfile-scope"),
                config("excluded-advise.yml"),
            ),
            (
                "excluded_admin_ignore",
                fixture("project-lockfile-scope"),
                config("excluded-ignore.yml"),
            ),
            (
                "pnpm12_env_tarball",
                saved_root("package-json-registry-only", "pnpm12-env-tarball"),
                lockfile.clone(),
            ),
            (
                "pnpm12_duplicate_tarball",
                saved_root("package-json-registry-only", "pnpm12-duplicate-tarball"),
                lockfile.clone(),
            ),
            (
                "pnpm12_project_git",
                saved_root("package-json-registry-only", "pnpm12-project-git"),
                lockfile.clone(),
            ),
            (
                "issue_1035_exotic",
                saved_root("pnpm12-issue-1035", "exotic"),
                lockfile.clone(),
            ),
            (
                "issue_1035_malformed",
                saved_root("pnpm12-issue-1035", "malformed"),
                lockfile.clone(),
            ),
            (
                "issue_1035_non_env",
                saved_root("pnpm12-issue-1035", "non-env"),
                lockfile.clone(),
            ),
            (
                "issue_1035_two_doc",
                saved_root("pnpm12-issue-1035", "two-doc"),
                lockfile.clone(),
            ),
            (
                "issue_1035_single_doc",
                saved_root("pnpm12-issue-1035", "single-doc"),
                lockfile,
            ),
        ],
    );
}
