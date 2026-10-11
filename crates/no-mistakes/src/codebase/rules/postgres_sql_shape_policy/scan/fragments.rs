use crate::codebase::postgres::prepared::PreparedSqlFragment;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::rules::RuleFinding;
use crate::fx::FxHashSet;

type Key = (String, usize, String, Option<String>);

#[derive(Default)]
pub(super) struct Findings {
    pending: Vec<(Option<Key>, RuleFinding)>,
}

impl Findings {
    pub(super) fn push(
        &mut self,
        fragment: &PreparedSqlFragment,
        site: SqlFactSite,
        source: Option<&str>,
        mut finding: RuleFinding,
    ) {
        let offset = fragment.physical_sites.get(&site).copied();
        if fragment.enumerated {
            finding.source_offset = offset;
            if let Some(source) = source {
                let line = if crate::codebase::ts_source::matching_disable_directive(
                    source,
                    Some(fragment.original_line),
                    &finding.rule,
                )
                .is_some()
                {
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
        self.pending
            .push((offset.map(|offset| key(&finding, offset)), finding));
    }

    pub(super) fn extend(self, output: &mut Vec<RuleFinding>) {
        // Complete statements provide the final physical line and executor
        // suppression context. Prefer them when the exact same fragment token
        // was also inspected independently; unrelated same-line tokens stay.
        let mut seen: FxHashSet<_> = output
            .iter()
            .filter_map(|finding| finding.source_offset.map(|offset| key(finding, offset)))
            .collect();
        for (key, finding) in self.pending {
            if let Some(key) = key {
                if seen.contains(&key) {
                    continue;
                }
                if finding.source_offset.is_some() {
                    seen.insert(key);
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
