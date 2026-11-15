use super::{
    accepts::DomainAccept, evaluate, possible_temporary, scalar_reads, target, Offender, ViewBounds,
};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundFact};

pub(in crate::codebase::rules::postgres_bounded_statements) struct Judgment {
    pub(in crate::codebase::rules::postgres_bounded_statements) offenders: Vec<Offender>,
    pub(in crate::codebase::rules::postgres_bounded_statements) accepts: Vec<DomainAccept>,
}

pub(in crate::codebase::rules::postgres_bounded_statements) fn judgment_with_views(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Judgment {
    judgment(fact, catalog, views, false)
}

pub(in crate::codebase::rules::postgres_bounded_statements) fn variant_judgment_with_views(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Judgment {
    judgment(fact, catalog, views, true)
}

/// The relations that make `fact` unbounded. A relation the catalog does not know is not
/// judged, and it bounds nothing either: it can supply every value of a column pinned to it.
#[cfg(any(test, feature = "test-instrumentation"))]
pub(in crate::codebase::rules::postgres_bounded_statements) fn offenders(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
) -> Vec<Offender> {
    judgment(fact, catalog, &ViewBounds::default(), false).offenders
}

pub(in crate::codebase::rules::postgres_bounded_statements) fn offenders_with_views(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Vec<Offender> {
    judgment(fact, catalog, views, false).offenders
}

pub(in crate::codebase::rules::postgres_bounded_statements) fn variant_offenders_with_views(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Vec<Offender> {
    judgment(fact, catalog, views, true).offenders
}

pub(super) fn judgment(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> Judgment {
    let temporary = possible_temporary::project(&fact.query, catalog);
    let query = scalar_reads::project(&temporary, fact.scalar_reads.as_ref(), catalog);
    let query = super::body_reads::project(&query, &fact.lateral_qualified_reads, catalog, views);
    let mut evaluation = evaluate(&query, catalog, views, precise_columns, fact.target);
    let offenders = match fact.target {
        Some(target_index) => target::offenders(&query, target_index, &evaluation, catalog, views),
        None => std::mem::take(&mut evaluation.offenders),
    };
    Judgment {
        offenders,
        accepts: evaluation.accepts,
    }
}
