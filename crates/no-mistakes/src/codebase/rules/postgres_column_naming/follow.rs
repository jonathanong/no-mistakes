use crate::codebase::postgres::SchemaCatalog;

pub(super) fn leads_to(
    catalog: &SchemaCatalog,
    table: &str,
    column: &str,
    targets: &[String],
) -> bool {
    let mut stack = Vec::new();
    walk(catalog, table, column, targets, &mut stack)
}

fn walk(
    catalog: &SchemaCatalog,
    table: &str,
    column: &str,
    targets: &[String],
    stack: &mut Vec<(String, String)>,
) -> bool {
    let pair = (table.to_string(), column.to_string());
    if stack.contains(&pair) {
        return false;
    }
    stack.push(pair);
    let found = catalog.table(table).is_some_and(|relation| {
        relation.foreign_keys.iter().any(|foreign_key| {
            let Some(position) = foreign_key.columns.iter().position(|name| name == column) else {
                return false;
            };
            if foreign_key.columns.len() == 1 {
                return targets
                    .iter()
                    .any(|target| target == &foreign_key.referenced_table);
            }
            if targets
                .iter()
                .any(|target| target == &foreign_key.referenced_table)
            {
                return true;
            }
            foreign_key
                .referenced_columns
                .get(position)
                .is_some_and(|referenced| {
                    walk(
                        catalog,
                        &foreign_key.referenced_table,
                        referenced,
                        targets,
                        stack,
                    )
                })
        })
    });
    stack.pop();
    found
}
