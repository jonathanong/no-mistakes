use super::*;

/// A subquery sizes the values it yields, unless it reads the row being checked: then every
/// row can find itself among them, whatever the subquery's own bound. A bare column that only
/// the catalog can place is kept with the pin.
pub(super) fn subquery_source(
    subquery: &Query,
    resolver: &Resolver,
    scope: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Sourced {
    let reads = resolver.reads(subquery);
    let query = query::bound_query(subquery, scope, positions);
    Sourced {
        source: if reads.certain {
            SqlPinSource::ReadQuery(query)
        } else {
            SqlPinSource::Query(query)
        },
        reads: reads.bare,
        qualified_reads: reads.qualified,
    }
}
