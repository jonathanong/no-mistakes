use super::{PostgresFindings, RuleFinding};
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::fx::{FxHashMap, FxHashSet};

type Identity = (String, usize, String, Option<String>);

#[derive(Default)]
pub(crate) struct VariantFindingDedup {
    seen: FxHashSet<Identity>,
    merged: FxHashMap<Identity, (usize, FxHashSet<String>)>,
}

impl VariantFindingDedup {
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
                let disabled_call = source.is_some_and(|source| {
                    crate::codebase::ts_source::matching_disable_directive(
                        source,
                        Some(locations.original_call_line),
                        &finding.rule,
                    )
                    .is_some()
                });
                let line = if disabled_call {
                    locations.original_call_line as usize
                } else {
                    position.source_line
                };
                move_line(&mut finding, line);
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
        move_line(&mut finding, line);
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

fn move_line(finding: &mut RuleFinding, line: usize) {
    let prefix = format!("{}:{}:", finding.file, finding.line);
    if let Some(message) = finding.message.strip_prefix(&prefix) {
        finding.message = format!("{}:{line}:{message}", finding.file);
    }
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
