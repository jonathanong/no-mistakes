use crate::codebase::postgres::idents::{
    ident_key, object_name_identity, relation_part_key, unwrap_expr,
};
use crate::codebase::postgres::{CatalogView, SchemaCatalog};
use sqlparser::ast::{
    Expr, SelectItem, SelectItemQualifiedWildcardKind, SetExpr, Statement, TableFactor,
    WildcardAdditionalOptions,
};
use std::collections::BTreeMap;

pub(super) fn mapped(
    catalog: &SchemaCatalog,
    view: &CatalogView,
    tables: &BTreeMap<&str, BTreeMap<String, bool>>,
) -> Option<BTreeMap<String, bool>> {
    let statements = crate::codebase::postgres::parse_postgres_sql(&view.definition).ok()?;
    let [Statement::Query(query)] = statements.as_slice() else {
        return None;
    };
    if query.with.is_some() || !query.pipe_operators.is_empty() {
        return None;
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return None;
    };
    let [from] = select.from.as_slice() else {
        return None;
    };
    if !from.joins.is_empty() {
        return None;
    }
    let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = &from.relation
    else {
        return None;
    };
    if alias
        .as_ref()
        .is_some_and(|alias| !alias.columns.is_empty())
    {
        return None;
    }
    let physical = object_name_identity(name);
    let table = catalog.bounded_relation(&physical)?;
    if !catalog.same_relation(&physical, &table.name) {
        return None;
    }
    let columns = tables.get(table.name.as_str())?;
    let qualifier = alias
        .as_ref()
        .map(|alias| relation_part_key(&ident_key(&alias.name)));
    let mut mapped = BTreeMap::new();
    let mut outputs = std::collections::BTreeSet::new();
    for item in &select.projection {
        match item {
            SelectItem::Wildcard(options) if plain_star(options) => {
                for (column, uuid) in columns {
                    if !outputs.insert(column.clone()) {
                        return None;
                    }
                    mapped.insert(column.clone(), *uuid);
                }
                continue;
            }
            SelectItem::QualifiedWildcard(
                SelectItemQualifiedWildcardKind::ObjectName(name),
                options,
            ) if plain_star(options) => {
                let owner = object_name_identity(name);
                let valid = qualifier.as_ref().map_or_else(
                    || catalog.same_relation(&owner, &table.name),
                    |alias| alias == &owner,
                );
                if !valid {
                    return None;
                }
                for (column, uuid) in columns {
                    if !outputs.insert(column.clone()) {
                        return None;
                    }
                    mapped.insert(column.clone(), *uuid);
                }
                continue;
            }
            _ => {}
        }
        let (expr, output) = match item {
            SelectItem::UnnamedExpr(expr) => (expr, None),
            SelectItem::ExprWithAlias { expr, alias } => (expr, Some(ident_key(alias))),
            _ => return None,
        };
        if let Some(output) = &output {
            if !outputs.insert(output.clone()) {
                return None;
            }
        }
        let parts = match unwrap_expr(expr) {
            Expr::Identifier(ident) => vec![ident_key(ident)],
            Expr::CompoundIdentifier(parts) => parts.iter().map(ident_key).collect(),
            _ => continue,
        };
        let column = parts.last()?;
        if output.is_none() && !outputs.insert(column.clone()) {
            return None;
        }
        if parts.len() > 1 {
            let owner = parts[..parts.len() - 1]
                .iter()
                .map(|part| relation_part_key(part))
                .collect::<Vec<_>>()
                .join(".");
            let valid = qualifier.as_ref().map_or_else(
                || catalog.same_relation(&owner, &table.name),
                |alias| alias == &owner,
            );
            if !valid {
                continue;
            }
        }
        if let Some(uuid) = columns.get(column) {
            let output = output.unwrap_or_else(|| column.clone());
            if mapped.insert(output, *uuid).is_some() {
                return None;
            }
        }
    }
    Some(mapped)
}

fn plain_star(options: &WildcardAdditionalOptions) -> bool {
    options.opt_ilike.is_none()
        && options.opt_exclude.is_none()
        && options.opt_except.is_none()
        && options.opt_replace.is_none()
        && options.opt_rename.is_none()
        && options.opt_alias.is_none()
}
