use super::{PostgresFindings, RuleFinding};
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

type Identity = (String, usize, String, Option<String>);

#[derive(Default)]
pub(crate) struct VariantFindingDedup {
    seen: FxHashSet<Identity>,
    merged: FxHashMap<Identity, (usize, FxHashSet<String>)>,
    /// Lets a finding on SQL recovered from another module name that module.
    root: Option<PathBuf>,
}

impl VariantFindingDedup {
    pub(crate) fn for_root(root: &Path) -> Self {
        Self {
            root: Some(root.to_path_buf()),
            ..Self::default()
        }
    }

    /// The module whose physical lines `position` counts, when it is not the
    /// executor's own file. Findings are attributed to that module so its
    /// suppression directives apply.
    fn foreign_file(
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

    pub(crate) fn push(
        &mut self,
        output: &mut PostgresFindings,
        facts: &SqlStatementFileFacts,
        site: SqlFactSite,
        source: Option<&str>,
        mut finding: RuleFinding,
    ) {
        if let Some(locations) = &facts.variant_locations {
            if let Some(position) = locations.position(site) {
                // A rule that has not named the module itself still carries the
                // executor's file, and `source` is then that file's text too.
                let retarget = self
                    .foreign_file(facts, position)
                    .filter(|file| *file != finding.file);
                let disabled_call = retarget.is_none()
                    && source.is_some_and(|source| {
                        crate::codebase::ts_source::matching_disable_directive(
                            source,
                            Some(
                                position
                                    .source_path
                                    .as_ref()
                                    .map_or(locations.original_call_line, |_| {
                                        position.source_line as u32
                                    }),
                            ),
                            &finding.rule,
                        )
                        .is_some()
                    });
                let line = if disabled_call && position.source_path.is_none() {
                    locations.original_call_line as usize
                } else {
                    position.source_line
                };
                move_line(&mut finding, retarget, line);
                // Keep disabled executions for suppression accounting, without
                // hiding another execution of the same SQL token.
                if !disabled_call && !self.keep_at(&finding, position.source_offset) {
                    return;
                }
                output.push_at(finding, position.source_offset);
                return;
            }
        }
        output.push(finding);
    }

    /// Preserve every semantic requirement while retaining one physical finding.
    /// This is used by consumers whose output buffer stays stable during scanning.
    pub(crate) fn push_merged(
        &mut self,
        output: &mut PostgresFindings,
        facts: &SqlStatementFileFacts,
        site: SqlFactSite,
        source: Option<&str>,
        mut finding: RuleFinding,
    ) {
        let position = facts.variant_locations.as_ref().and_then(|locations| {
            locations.position(site.clone()).and_then(|position| {
                let disabled = source.is_some_and(|source| {
                    crate::codebase::ts_source::matching_disable_directive(
                        source,
                        Some(locations.original_call_line),
                        &finding.rule,
                    )
                    .is_some()
                });
                position
                    .source_offset
                    .filter(|_| !disabled)
                    .map(|offset| (offset, position.source_line))
            })
        });
        let Some((offset, line)) = position else {
            self.push(output, facts, site, source, finding);
            return;
        };
        move_line(&mut finding, None, line);
        let identity = identity(&finding, offset);
        if let Some((index, messages)) = self.merged.get_mut(&identity) {
            let original = output.get_mut(*index);
            if messages.insert(finding.message.clone()) {
                let prefix = format!("{}:{}: ", finding.file, finding.line);
                let message = finding
                    .message
                    .strip_prefix(&prefix)
                    .unwrap_or(&finding.message);
                original.message.push_str("; ");
                original.message.push_str(message);
            }
            if original.import != finding.import {
                original.import = None;
            }
            return;
        }
        if self.keep_at(&finding, Some(offset)) {
            let messages = FxHashSet::from_iter([finding.message.clone()]);
            self.merged.insert(identity, (output.len(), messages));
            output.push_at(finding, Some(offset));
        }
    }

    pub(crate) fn keep_at(&mut self, finding: &RuleFinding, offset: Option<usize>) -> bool {
        offset.is_none_or(|offset| self.seen.insert(identity(finding, offset)))
    }
}

fn identity(finding: &RuleFinding, offset: usize) -> Identity {
    (
        finding.file.clone(),
        offset,
        finding.rule.clone(),
        finding.target.clone(),
    )
}

fn move_line(finding: &mut RuleFinding, file: Option<String>, line: usize) {
    let prefix = format!("{}:{}:", finding.file, finding.line);
    let file = file.unwrap_or_else(|| finding.file.clone());
    if let Some(message) = finding.message.strip_prefix(&prefix) {
        finding.message = format!("{file}:{line}:{message}");
    }
    finding.file = file;
    finding.line = line;
}

pub(crate) fn index_sql_variants(
    statements: &[SqlStatementFileFacts],
) -> FxHashMap<(usize, usize), &SqlStatementFileFacts> {
    statements
        .iter()
        .filter_map(|statement| {
            statement
                .variant_locations
                .as_ref()
                .map(|locations| ((locations.call_index, locations.variant_index), statement))
        })
        .collect()
}
