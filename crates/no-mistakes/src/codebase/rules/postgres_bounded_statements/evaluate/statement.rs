use super::{evaluate, possible_temporary, scalar_reads, target, Offender, ViewBounds};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundFact};

pub(super) fn offenders_with_columns(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> Vec<Offender> {
    let temporary = possible_temporary::project(&fact.query, catalog);
    let query = scalar_reads::project(&temporary, fact.scalar_reads.as_ref(), catalog);
    let evaluation = evaluate(&query, catalog, views, precise_columns, fact.target);
    let Some(target) = fact.target else {
        return evaluation.offenders;
    };
    target::offenders(&query, target, &evaluation, catalog, views)
}
