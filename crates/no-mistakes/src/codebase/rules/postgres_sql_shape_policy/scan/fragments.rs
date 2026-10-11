use crate::codebase::postgres::prepared::PreparedSqlFragment;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::rules::RuleFinding;
use crate::fx::{FxHashMap, FxHashSet};

type Key = (String, usize, String, Option<String>);

#[derive(Default)]
pub(super) struct Findings {
    pending: Vec<(Option<Key>, Option<u32>, RuleFinding)>,
    active_complete: FxHashSet<Key>,
    disabled_complete: FxHashMap<Key, FxHashSet<u32>>,
}

impl Findings {
    pub(super) fn push_complete(
        &mut self,
        output: &mut Vec<RuleFinding>,
        facts: &crate::codebase::postgres::SqlStatementFileFacts,
        site: SqlFactSite,
        source: Option<&str>,
        finding: RuleFinding,
        dedup: &mut crate::codebase::rules::VariantFindingDedup,
    ) {
        if let Some(locations) = &facts.variant_locations {
            if let Some(offset) = locations
                .position(site.clone())
                .and_then(|position| position.source_offset)
            {
                let identity = key(&finding, offset);
                let disabled = source.is_some_and(|source| {
                    crate::codebase::ts_source::matching_disable_directive(
                        source,
                        Some(locations.original_call_line),
                        &finding.rule,
                    )
                    .is_some()
                });
                if disabled {
                    self.disabled_complete
                        .entry(identity)
                        .or_default()
                        .extend(locations.append_sites.iter().copied());
                } else {
                    self.active_complete.insert(identity);
                }
            }
        }
        dedup.push(output, facts, site, source, finding);
    }

    pub(super) fn push(
        &mut self,
        fragment: &PreparedSqlFragment,
        site: SqlFactSite,
        source: Option<&str>,
        mut finding: RuleFinding,
    ) {
        let offset = fragment.physical_sites.get(&site).copied();
        let disabled_host = fragment.enumerated
            && source.is_some_and(|source| {
                crate::codebase::ts_source::matching_disable_directive(
                    source,
                    Some(fragment.original_line),
                    &finding.rule,
                )
                .is_some()
            });
        if fragment.enumerated {
            finding.source_offset = offset;
            if let Some(source) = source {
                let line = if disabled_host {
                    fragment.original_line as usize
                } else {
                    offset.map_or(finding.line, |offset| {
                        crate::codebase::ts_source::byte_offset_to_line(source, offset) as usize
                    })
                };
                let prefix = format!("{}:{}:", finding.file, finding.line);
                // Every finding here comes from scan::finding's file:line prefix.
                finding.message = format!(
                    "{}:{line}:{}",
                    finding.file,
                    &finding.message[prefix.len()..]
                );
                finding.line = line;
            }
        }
        self.pending.push((
            offset
                .filter(|_| !disabled_host)
                .map(|offset| key(&finding, offset)),
            fragment.append_site,
            finding,
        ));
    }

    pub(super) fn extend(self, output: &mut Vec<RuleFinding>) {
        // Active executions deduplicate shared physical tokens globally. A
        // disabled execution owns only the append sites that contributed its
        // SQL, so an unrelated active builder remains independently checked.
        let mut seen = self.active_complete;
        for (identity, append_site, finding) in self.pending {
            if let Some(identity) = identity {
                let owned_disabled = append_site.is_some_and(|site| {
                    self.disabled_complete
                        .get(&identity)
                        .is_some_and(|sites| sites.contains(&site))
                });
                if seen.contains(&identity) || owned_disabled {
                    continue;
                }
                if finding.source_offset.is_some() {
                    seen.insert(identity);
                }
            }
            output.push(finding);
        }
    }
}

fn key(finding: &RuleFinding, offset: usize) -> Key {
    (
        finding.file.clone(),
        offset,
        finding.rule.clone(),
        finding.target.clone(),
    )
}
