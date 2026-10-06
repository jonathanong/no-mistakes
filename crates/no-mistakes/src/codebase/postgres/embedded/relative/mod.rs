mod package;
mod project;
mod types;

pub(crate) use package::{package_name, package_root_for_specifier};
pub(crate) use project::project_relative_scoped_facts;
pub(crate) use types::{
    PendingRelativeCall, PendingRelativeScope, PendingRelativeSpan, RelativeScopedCandidate,
};

#[cfg(test)]
mod tests;
