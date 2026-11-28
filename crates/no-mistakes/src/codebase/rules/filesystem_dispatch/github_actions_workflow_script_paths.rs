mod composites;
mod diagnostics;
mod emit;
mod extract;
pub(in crate::codebase::rules) mod options;
mod scripts;
mod shell_cmd;
mod triggers;
mod workflow;

pub const RULE_ID: &str = "github-actions-workflow-script-paths";

pub use diagnostics::{check, diagnostics_with_files_and_sources};
pub(crate) use diagnostics::{check_with_files, check_with_files_sources_and_deferred_suppression};

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod review_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
