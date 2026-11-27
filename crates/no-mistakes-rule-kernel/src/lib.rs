//! Shared rule mechanics. The facade supplies request data and registers rules.

pub mod file_diagnostics;
pub mod finding;
pub mod path_filter;
pub mod rule_context;
pub mod rule_ids;
pub mod run;
pub mod source_access;
pub mod suppression;
pub mod validate_options;

#[doc(hidden)]
pub mod paths;
