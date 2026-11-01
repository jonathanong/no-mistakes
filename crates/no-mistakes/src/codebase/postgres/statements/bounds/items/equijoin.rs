//! Copy a caller-sized pin onto a derived item so its base columns can carry the pin inward.
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundPin, SqlPinSource,
};

pub(super) fn imply(items: &mut [SqlBoundItem]) {
    let snapshot: Vec<Vec<SqlBoundPin>> = items.iter().map(|item| item.pins.clone()).collect();
    for (index, item) in items.iter_mut().enumerate() {
        // Only a derived item pushes pins inward. A copied value on a base table is a new proof.
        if !matches!(item.kind, SqlBoundItemKind::Query(_)) {
            continue;
        }
        let mut extra = Vec::new();
        for pin in &snapshot[index] {
            let SqlPinSource::Items(sources) = &pin.source else {
                continue;
            };
            let [source] = sources.as_slice() else {
                continue;
            };
            let Some(peer_column) = &pin.peer_column else {
                continue;
            };
            let Some(peer_pins) = snapshot.get(*source) else {
                continue;
            };
            for peer in peer_pins {
                if &peer.column != peer_column || !portable(peer) {
                    continue;
                }
                let Some(alternative) = copied_alternative(pin.alternative, peer.alternative)
                else {
                    continue;
                };
                let mut copy = peer.clone();
                copy.column = pin.column.clone();
                copy.alternative = alternative;
                copy.peer_column = None;
                copy.unqualified_candidate = false;
                if !item.pins.contains(&copy) && !extra.contains(&copy) {
                    extra.push(copy);
                }
            }
        }
        item.pins.extend(extra);
    }
}

/// Item indexes belong to this statement. Only a self-contained source can cross the join.
fn portable(pin: &SqlBoundPin) -> bool {
    matches!(
        pin.source,
        SqlPinSource::Value | SqlPinSource::Query(_) | SqlPinSource::ArrayQuery(_)
    )
}

/// `None` skips the copy. `Some` is the alternative the copied pin belongs to.
fn copied_alternative(join: Option<usize>, peer: Option<usize>) -> Option<Option<usize>> {
    match (join, peer) {
        (None, peer) => Some(peer),
        (Some(branch), None) => Some(Some(branch)),
        (Some(branch), Some(other)) if branch == other => Some(Some(branch)),
        (Some(_), Some(_)) => None,
    }
}
