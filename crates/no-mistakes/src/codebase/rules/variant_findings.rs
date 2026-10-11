use super::RuleFinding;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(crate) struct VariantFindingDedup {
    seen: FxHashSet<(String, usize, String, Option<String>)>,
}

impl VariantFindingDedup {
    pub(crate) fn push(
        &mut self,
        output: &mut Vec<RuleFinding>,
        facts: &SqlStatementFileFacts,
        site: SqlFactSite,
        source: Option<&str>,
        mut finding: RuleFinding,
    ) {
        if let Some(locations) = &facts.variant_locations {
            if let Some(position) = locations.position(site) {
                let line = source
                    .filter(|source| {
                        crate::codebase::ts_source::matching_disable_directive(
                            source,
                            Some(locations.original_call_line),
                            &finding.rule,
                        )
                        .is_some()
                    })
                    .map_or(position.source_line, |_| {
                        locations.original_call_line as usize
                    });
                move_line(&mut finding, line);
                finding.source_offset = position.source_offset;
                if !self.keep_at(&finding, position.source_offset) {
                    return;
                }
            }
        }
        output.push(finding);
    }

    pub(crate) fn keep_at(&mut self, finding: &RuleFinding, offset: Option<usize>) -> bool {
        offset.is_none_or(|offset| {
            self.seen.insert((
                finding.file.clone(),
                offset,
                finding.rule.clone(),
                finding.target.clone(),
            ))
        })
    }
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
