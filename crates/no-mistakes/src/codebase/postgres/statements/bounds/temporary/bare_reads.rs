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

fn scalar(fact: &mut crate::codebase::postgres::SqlBoundFact, state: &State) {
    if let Some(scalar) = &mut fact.scalar_reads {
        let known = |name: &str| !state.contains(name) && state.possible_temporary(name).is_none();
        scalar.outer_tables.retain(|(name, _)| known(name));
        scalar
            .reads
            .retain(|read| read.tables.iter().all(|name| known(name)));
    }
}

impl super::TemporaryRelations {
    pub(super) fn project_fact(&self, fact: &mut crate::codebase::postgres::SqlBoundFact) {
        scalar(fact, &self.state);
        self.query(&mut fact.query);
    }
}
