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
