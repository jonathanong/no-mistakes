#[path = "../test_support/git.rs"]
mod git;
#[path = "../test_support/gitignore_fixture.rs"]
mod gitignore_fixture;
#[path = "../test_support/policy_advisories.rs"]
mod policy_advisories;

pub(crate) use git::{git_add_all, git_init};
pub(crate) use gitignore_fixture::materialize_gitignore_fixture;
#[cfg(any(test, feature = "test-instrumentation"))]
pub(crate) use gitignore_fixture::materialize_saved_fixture;
pub(crate) use policy_advisories::without_hygiene_advisories;

pub(crate) fn rule_fixture_root(rule: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("fixture")
}
