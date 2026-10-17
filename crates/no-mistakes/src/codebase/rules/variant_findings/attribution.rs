//! Where a finding on SQL recovered from another module is reported, and which
//! caller-side directives still suppress it.

use super::VariantFindingDedup;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::rules::RuleFinding;

impl<'a> VariantFindingDedup<'a> {
    /// The caller-side line whose directive suppresses `rule`, when SQL
    /// recovered from another module is executed by a call that opts out. The
    /// executor call and each imported-helper call count, as does a file-level
    /// opt-out, so moving a finding to the helper never removes a suppression.
    pub(super) fn caller_suppression(
        &self,
        facts: &SqlStatementFileFacts,
        locations: &crate::codebase::postgres::statements::SqlVariantLocations,
        rule: &str,
    ) -> Option<(String, usize)> {
        let (root, sources) = (self.root.as_deref()?, self.sources?);
        let source = crate::codebase::rules::read_source(sources, &facts.path)?;
        let lines = std::iter::once(locations.original_call_line)
            .chain(locations.caller_lines.iter().copied());
        let line = lines.into_iter().find(|line| {
            crate::codebase::ts_source::matching_disable_directive(&source, Some(*line), rule)
                .is_some()
        })?;
        let file = crate::codebase::ts_source::relative_slash_path(root, &facts.path);
        Some((file, line as usize))
    }

    /// The module whose physical lines `position` counts, when it is not the
    /// executor's own file. Findings are attributed to that module so its
    /// suppression directives apply.
    pub(super) fn foreign_file(
        &self,
        facts: &SqlStatementFileFacts,
        position: &crate::codebase::postgres::statements::SqlFactPosition,
    ) -> Option<String> {
        let path = position
            .source_path
            .as_ref()
            .filter(|path| **path != facts.path)?;
        let root = self.root.as_deref()?;
        Some(crate::codebase::ts_source::relative_slash_path(root, path))
    }
}

pub(super) fn move_line(finding: &mut RuleFinding, file: Option<String>, line: usize) {
    let prefix = format!("{}:{}:", finding.file, finding.line);
    let file = file.unwrap_or_else(|| finding.file.clone());
    if let Some(message) = finding.message.strip_prefix(&prefix) {
        finding.message = format!("{file}:{line}:{message}");
    }
    finding.file = file;
    finding.line = line;
}
