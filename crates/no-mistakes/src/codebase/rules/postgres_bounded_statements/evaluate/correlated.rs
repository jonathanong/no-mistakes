use super::super::view_bounds::ViewBounds;
use super::{lateral, Evaluation};
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundPin, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::SchemaCatalog;
use std::borrow::Cow;

mod project;
pub(super) use project::project;

pub(super) fn nested(
    item: &SqlBoundItem,
    preceding: &[SqlBoundItem],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> Option<Evaluation> {
    let SqlBoundItemKind::Query(inner) = &item.kind else {
        return None;
    };
    let projected = if lateral::dependencies(item, preceding, catalog).is_some() {
        project(inner, &item.lateral_qualified_reads, catalog, views)
    } else {
        Cow::Borrowed(inner)
    };
    Some(super::evaluate(
        &projected,
        catalog,
        views,
        precise_columns,
        None,
    ))
}

pub(super) fn pin_query(
    inner: &SqlBoundQuery,
    pin: &SqlBoundPin,
    outer: &[SqlBoundItem],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> Evaluation {
    let projected = if !pin.qualified_reads.is_empty()
        && !matches!(pin.source, SqlPinSource::ReadQuery(_))
        && lateral::qualified_dependencies(&pin.qualified_reads, outer, catalog).is_some()
    {
        project(inner, &pin.qualified_reads, catalog, views)
    } else {
        Cow::Borrowed(inner)
    };
    super::evaluate(&projected, catalog, views, precise_columns, None)
}
