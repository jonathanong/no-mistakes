use super::super::{
    evidence,
    expressions::ident,
    scope::{definitions, CatalogScope},
    union,
};
use sqlparser::ast::Query;
use std::collections::BTreeMap;

type Projections = BTreeMap<String, union::Columns>;
pub(super) type Overrides = BTreeMap<usize, Projections>;

pub(super) fn projections(
    query: &Query,
    scope: &CatalogScope<'_>,
    recursive: bool,
    mut visible: Projections,
    names: &definitions::Names,
    overrides: &mut Overrides,
) -> Projections {
    let Some(with) = &query.with else {
        return visible;
    };
    if recursive {
        for cte in &with.cte_tables {
            visible.remove(&ident(&cte.alias.name));
        }
    }
    for cte in &with.cte_tables {
        overrides.insert(definitions::key(&cte.query), visible.clone());
        let name = ident(&cte.alias.name);
        visible.remove(&name);
        if !recursive {
            let definition_scope = CatalogScope {
                catalog: scope.catalog,
                parameters: scope.parameters,
                shadowed: names
                    .get(&definitions::key(&cte.query))
                    .expect("CTE definition has a lexical scope"),
            };
            if let Some(columns) = union::columns(&cte.query)
                .filter(|_| evidence::catalogued(cte.query.as_ref(), &definition_scope))
            {
                visible.insert(name, columns);
            }
        }
    }
    visible
}
