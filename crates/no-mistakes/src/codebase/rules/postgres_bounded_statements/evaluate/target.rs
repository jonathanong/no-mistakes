use super::super::view_bounds::ViewBounds;
use super::{driven, reads::relation_offender, Evaluation, Offender};
use crate::codebase::postgres::{
    SchemaCatalog, SqlBoundInputMode, SqlBoundItemKind, SqlBoundQuery,
};

pub(super) fn offenders(
    query: &SqlBoundQuery,
    target: usize,
    evaluation: &Evaluation,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Vec<Offender> {
    if super::reads::proven_capped(query, catalog) || evaluation.items[target] {
        return Vec::new();
    }
    if let Some(roots) = driven::roots(query, target, &evaluation.offenders, catalog) {
        return roots;
    }
    let item = &query.items[target];
    match &item.kind {
        SqlBoundItemKind::Table(name) => {
            relation_offender(name, item.line, item.column, catalog, views)
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}

pub(super) fn project(
    query: &SqlBoundQuery,
    mut evaluation: Evaluation,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Evaluation {
    if query.input_mode == SqlBoundInputMode::Returning {
        evaluation.bounded = super::reads::proven_capped(query, catalog) || evaluation.items[0];
        evaluation.offenders = offenders(query, 0, &evaluation, catalog, views);
    }
    evaluation
}
