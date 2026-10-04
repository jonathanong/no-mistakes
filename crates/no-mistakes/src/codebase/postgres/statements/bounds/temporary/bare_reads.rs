use super::State;
use crate::codebase::postgres::SqlBoundItem;

pub(super) fn project(item: &mut SqlBoundItem, state: &State) {
    // A temporary namesake has an unknown column layout: permanent catalog columns
    // cannot prove that a bare reference stays local instead of resolving outward.
    for read in item
        .lateral_reads
        .iter_mut()
        .chain(item.pins.iter_mut().flat_map(|pin| pin.reads.iter_mut()))
    {
        read.tables
            .retain(|name| state.possible_temporary(name).is_none());
    }
}
