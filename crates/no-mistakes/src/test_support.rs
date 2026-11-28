#[cfg(test)]
pub(crate) mod allocations;
mod check_json;
mod git;
mod git_extended;
mod gitignore_fixture;
mod js_string;
mod workflow_topology_impact;

pub use check_json::{check_json, CheckJsonArg};
pub use git::{git_add_all, git_init};
pub use git_extended::{git_add_force, git_commit_all, git_config, git_skip_worktree};
pub use gitignore_fixture::{materialize_gitignore_fixture, materialize_saved_fixture};
pub use js_string::{
    js_string_literal, materialize_quote_containing_fixture, replace_quoted_placeholder,
};
pub use workflow_topology_impact::materialize_workflow_topology_impact_fixture;

pub fn fixture_root(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

pub fn rule_fixture_root(rule: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("fixture")
}

mod policy_advisories;
pub use policy_advisories::{without_hygiene_advisories, without_hygiene_advisory_json};
