use super::RULE_ID;
use crate::codebase::rules::postgres_findings::site_identity::SiteIdentities;
use crate::codebase::rules::{LocatedRuleFinding, PostgresFindings};
use crate::codebase::ts_source::{matching_disable_directive, SourceStore};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

type Coordinates = (usize, usize);
type Physical = (String, usize);
type Execution = (String, bool, usize);
pub(super) type Site = (
    String,
    Option<usize>,
    Coordinates,
    Option<Execution>,
    Vec<u32>,
);

#[derive(Default)]
pub(super) struct Groups {
    sites: BTreeMap<Site, Group>,
    occurrences: BTreeMap<(Physical, Vec<u32>), BTreeMap<Coordinates, usize>>,
    identities: SiteIdentities<(Physical, Option<Execution>)>,
    projection: usize,
}

struct Group {
    position: (usize, usize),
    members: Vec<LocatedRuleFinding>,
}

impl Groups {
    pub(super) fn begin_projection(&mut self) {
        self.occurrences.clear();
        self.projection += 1;
    }

    pub(super) fn push(&mut self, mut site: Site, finding: LocatedRuleFinding) {
        let position = site.2;
        if let Some(offset) = site.1 {
            let physical = (site.0.clone(), offset);
            let occurrences = self
                .occurrences
                .entry((physical.clone(), site.4.clone()))
                .or_default();
            let next = occurrences.len();
            let occurrence = *occurrences.entry(position).or_insert(next);
            self.identities.record(
                (physical, site.3.clone()),
                self.projection,
                (site.4.clone(), occurrence),
            );
            site.2 = (0, occurrence);
        }
        let group = self.sites.entry(site).or_insert_with(|| Group {
            position,
            members: Vec::new(),
        });
        group.position = group.position.min(position);
        group.members.push(finding);
    }

    pub(super) fn finish(self, root: &Path, sources: &SourceStore) -> PostgresFindings {
        let mut output = PostgresFindings::default();
        let mut grouped = Vec::new();
        let colors = self.identities.finish();
        let mut sites = BTreeMap::<Site, Group>::new();
        for (mut site, group) in self.sites {
            if let Some(offset) = site.1 {
                let scope = ((site.0.clone(), offset), site.3.clone());
                site.2 = (0, colors[&scope][&(std::mem::take(&mut site.4), site.2 .1)]);
                site.3 = None;
            }
            let merged = sites.entry(site).or_insert_with(|| Group {
                position: group.position,
                members: Vec::new(),
            });
            merged.position = merged.position.min(group.position);
            merged.members.extend(group.members);
        }
        for ((file, _, _, _, _), group) in sites {
            if let Some(finding) = merge(group.members, root, sources, &mut output) {
                grouped.push((file, group.position, finding));
            }
        }
        if grouped.len() < 2 {
            output.extend(grouped.into_iter().map(|(_, _, finding)| finding));
            return output;
        }
        let suffixes = crate::codebase::rules::postgres_findings::site_suffixes(
            grouped
                .iter()
                .map(|(file, position, located)| (&located.finding, file.as_str(), *position)),
            "expression",
        );
        for ((_, _, mut located), suffix) in grouped.into_iter().zip(suffixes) {
            if let Some(suffix) = suffix {
                located.finding.message.push_str(&suffix);
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
