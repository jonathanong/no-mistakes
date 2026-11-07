use super::RULE_ID;
use crate::codebase::rules::{LocatedRuleFinding, PostgresFindings};
use crate::codebase::ts_source::{matching_disable_directive, SourceStore};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) type Site = (String, Option<usize>, (usize, usize), Option<(bool, usize)>);

#[derive(Default)]
pub(super) struct Groups(BTreeMap<Site, Group>);

struct Group {
    position: (usize, usize),
    call: Option<(bool, usize)>,
    members: Vec<LocatedRuleFinding>,
}

impl Groups {
    pub(super) fn push(&mut self, mut site: Site, finding: LocatedRuleFinding) {
        let position = site.2;
        let call = site.3;
        if site.1.is_some() {
            site.2 = (0, 0);
            site.3 = None;
        }
        let group = self.0.entry(site).or_insert_with(|| Group {
            position,
            call,
            members: Vec::new(),
        });
        group.position = group.position.min(position);
        group.call = group.call.min(call);
        group.members.push(finding);
    }

    pub(super) fn finish(self, root: &Path, sources: &SourceStore) -> PostgresFindings {
        let mut output = PostgresFindings::default();
        let mut grouped = Vec::new();
        for ((file, _, _, _), group) in self.0 {
            if let Some(finding) = merge(group.members, root, sources, &mut output) {
                grouped.push((file, group.position, group.call, finding));
            }
        }
        if grouped.len() < 2 {
            output.extend(grouped.into_iter().map(|(_, _, _, finding)| finding));
            return output;
        }
        let mut counts = BTreeMap::new();
        for (_, _, _, located) in &grouped {
            *counts.entry(located.finding.clone()).or_insert(0usize) += 1;
        }
        for (file, position, call, mut located) in grouped {
            if counts[&located.finding] > 1 {
                let call = call
                    .map(|(variant, index)| {
                        if variant {
                            format!("; executor call {index}")
                        } else {
                            format!("; execution {index}")
                        }
                    })
                    .unwrap_or_default();
                located.finding.message.push_str(&format!(
                    " (expression at SQL {}:{} from {file}{call})",
                    position.0, position.1
                ));
            }
            output.0.push(located);
        }
        output
    }
}

fn merge(
    members: Vec<LocatedRuleFinding>,
    root: &Path,
    sources: &SourceStore,
    output: &mut PostgresFindings,
) -> Option<LocatedRuleFinding> {
    let mut active = Vec::new();
    for located in members {
        let finding = &located.finding;
        let source = crate::codebase::rules::read_source(sources, &root.join(&finding.file));
        let disabled = source.as_deref().is_some_and(|source| {
            matching_disable_directive(source, u32::try_from(finding.line).ok(), RULE_ID).is_some()
        });
        if disabled {
            output.0.push(located);
        } else {
            active.push(located);
        }
    }
    if active.len() < 2 {
        return active.pop();
    }
    active.sort();
    let mut finding = active[0].clone();
    let columns: BTreeSet<_> = active
        .iter()
        .filter_map(|located| located.finding.target.as_deref())
        .collect();
    if columns.len() > 1 {
        let target = columns.into_iter().collect::<Vec<_>>().join(", ");
        let original = finding.finding.target.as_deref().unwrap();
        finding.finding.message = finding.finding.message.replacen(
            &format!("UUID column {original};"),
            &format!("UUID columns {target};"),
            1,
        );
        finding.finding.target = Some(target);
    }
    Some(finding)
}
