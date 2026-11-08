use super::{site, Group, StatementKey};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

pub(super) fn coalesce(
    groups: BTreeMap<StatementKey, Group>,
    positions: site::Positions,
) -> BTreeMap<StatementKey, Group> {
    let colors = positions.colors();
    let mut normalized = BTreeMap::<StatementKey, Group>::new();
    for ((file, call, position), group) in groups {
        let position = match position {
            site::Position::Source(physical, invocation, occurrence) => {
                let scope = (file.clone(), call, physical.clone());
                let color = colors[&scope][&(invocation, occurrence)];
                site::Position::Source(physical, Vec::new(), color)
            }
            position => position,
        };
        match normalized.entry((file, call, position)) {
            Entry::Vacant(entry) => {
                entry.insert(group);
            }
            Entry::Occupied(mut entry) => {
                let merged = entry.get_mut();
                merged.findings.extend(group.findings);
                merged.spans.extend(group.spans);
                merged.at = merged.at.min(group.at);
            }
        }
    }
    normalized
}
