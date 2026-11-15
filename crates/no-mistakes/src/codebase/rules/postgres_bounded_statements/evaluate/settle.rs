//! Close item bounds, then collect offenders against the resulting row cap.
use super::super::view_bounds::ViewBounds;
use super::keys::Keys;
use super::{propagation, reads, singleton, Evaluation, Offender};
use crate::codebase::postgres::statements::SqlBoundQuery;
use crate::codebase::postgres::SchemaCatalog;

pub(super) struct Inputs<'a> {
    pub query: &'a SqlBoundQuery,
    pub keys: &'a [Keys],
    pub nested: &'a [Option<Evaluation>],
    pub pin_subqueries: &'a [Vec<Option<Evaluation>>],
    pub catalog: &'a SchemaCatalog,
    pub views: &'a ViewBounds,
    pub precise_columns: bool,
}

pub(super) fn finish(inputs: &Inputs<'_>, bounded: &mut [bool]) -> (bool, Vec<Offender>) {
    propagation::bound(inputs.keys, bounded);
    singleton::finish(inputs.query, inputs.catalog, inputs.keys, bounded);
    let offenders = reads::collect(
        inputs.query,
        inputs.nested,
        inputs.pin_subqueries,
        bounded,
        inputs.catalog,
        inputs.views,
        inputs.precise_columns,
    );
    (
        reads::row_capped(inputs.query, bounded) || bounded.iter().all(|state| *state),
        offenders,
    )
}
