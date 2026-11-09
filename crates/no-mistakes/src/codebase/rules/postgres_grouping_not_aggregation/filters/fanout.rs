use super::super::unique;
use super::*;
pub(super) fn fanout(
    select: &Select,
    catalog: &super::super::scope::CatalogScope<'_>,
    order_owners: &BTreeSet<String>,
) -> bool {
    let Some(relations) = walk::table_columns(select) else {
        return false;
    };
    if relations.len() < 2
        || relations
            .iter()
            .any(|r| catalog.relation(&r.name).is_none())
    {
        return false;
    }
    let Some(projected) = projection(select) else {
        return false;
    };
    if unique::redundant(select, catalog) {
        return false;
    }
    let driver = &relations[0];
    if relations[1..].iter().any(|relation| {
        order_owners.contains(&relation.alias)
            || catalog.relation(&relation.name).is_some_and(|table| {
                table
                    .columns
                    .iter()
                    .any(|column| order_owners.contains(&format!(":{}", column.name)))
            })
    }) {
        return false;
    }
    let mut count = false;
    for expr in projected {
        if let Some(function) = function(expr, "count") {
            let Some(arg) = argument(function) else {
                return false;
            };
            let mut keys = vec![arg];
            keys.extend(groups(select).iter());
            if args(function)
                .is_none_or(|a| a.duplicate_treatment != Some(DuplicateTreatment::Distinct))
                || !unique::key_covers(
                    driver,
                    &keys,
                    select,
                    catalog,
                    &BTreeSet::new(),
                    (false, Some(arg)),
                )
            {
                return false;
            }
            count = true;
        } else if aggregate(expr) || !groups(select).iter().any(|key| bare(key) == bare(expr)) {
            return false;
        }
        if relations[1..].iter().any(|relation| {
            catalog.relation(&relation.name).is_some_and(|table| {
                uses_output(
                    expr,
                    &relation.alias,
                    &table
                        .columns
                        .iter()
                        .map(|column| column.name.clone())
                        .collect::<Vec<_>>(),
                )
            })
        }) {
            return false;
        }
    }
    count
        && groups(select).iter().all(|expr| {
            relations[1..].iter().all(|relation| {
                !catalog.relation(&relation.name).is_some_and(|table| {
                    uses_output(
                        expr,
                        &relation.alias,
                        &table
                            .columns
                            .iter()
                            .map(|column| column.name.clone())
                            .collect::<Vec<_>>(),
                    )
                })
            })
        })
        && select.having.is_none()
}
