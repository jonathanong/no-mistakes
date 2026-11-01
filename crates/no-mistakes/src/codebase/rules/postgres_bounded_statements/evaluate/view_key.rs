//! A view key exists only when the definition projects one base primary key 1:1.
mod join;
use crate::codebase::postgres::idents::{
    ident_key, object_name_ident, object_name_key, unwrap_expr,
};
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{
    Expr, GroupByExpr, ObjectName, ObjectNamePart, Select, SelectItem,
    SelectItemQualifiedWildcardKind, SetExpr, Statement, TableAlias, TableFactor,
    WildcardAdditionalOptions,
};

pub(in crate::codebase::rules::postgres_bounded_statements) fn visible_key(
    catalog: &SchemaCatalog,
    name: &str,
) -> Option<Vec<String>> {
    let view = catalog.view_relation(name)?;
    let statements = crate::codebase::postgres::parse_postgres_sql(&view.definition).ok()?;
    let [Statement::Query(query)] = statements.as_slice() else {
        return None;
    };
    if query.with.is_some()
        || query.limit_clause.is_some()
        || query.fetch.is_some()
        || !query.pipe_operators.is_empty()
    {
        return None;
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return None;
    };
    if !one_relation(select) {
        return None;
    }
    let (base_quals, primary_key, bare_columns) = base_key(select, catalog)?;
    projected(select, &base_quals, &primary_key, bare_columns)
}

fn base_key(select: &Select, catalog: &SchemaCatalog) -> Option<(Vec<String>, Vec<String>, bool)> {
    let from = select.from.first()?;
    let (base_name, base_quals) = plain_table(&from.relation)?;
    if catalog.view_relation(&base_name).is_some() {
        return None;
    }
    let primary_key = catalog
        .bounded_relation(&base_name)?
        .primary_key
        .clone()
        .filter(|key| !key.is_empty())?;
    if !from
        .joins
        .iter()
        .all(|join| join::preserves_rows(join, &base_quals, catalog))
    {
        return None;
    }
    Some((base_quals, primary_key, from.joins.is_empty()))
}

fn one_relation(select: &Select) -> bool {
    let grouped = match &select.group_by {
        GroupByExpr::Expressions(items, modifiers) => !items.is_empty() || !modifiers.is_empty(),
        GroupByExpr::All(_) => true,
    };
    !grouped
        && select.distinct.is_none()
        && select.top.is_none()
        && select.having.is_none()
        && select.qualify.is_none()
        && select.lateral_views.is_empty()
        && select.connect_by.is_empty()
        && select.from.len() == 1
}

pub(super) fn plain_table(factor: &TableFactor) -> Option<(String, Vec<String>)> {
    let TableFactor::Table {
        name,
        alias,
        args: None,
        with_hints,
        version: None,
        with_ordinality: false,
        partitions,
        json_path: None,
        sample: None,
        index_hints,
    } = factor
    else {
        return None;
    };
    if !with_hints.is_empty()
        || !partitions.is_empty()
        || !index_hints.is_empty()
        || alias
            .as_ref()
            .is_some_and(|alias| !alias.columns.is_empty() || alias.at.is_some())
        || name
            .0
            .iter()
            .any(|part| !matches!(part, ObjectNamePart::Identifier(_)))
    {
        return None;
    }
    let key = object_name_key(name);
    (!key.is_empty()).then(|| (key, qualifiers(name, alias.as_ref())))
}

fn qualifiers(name: &ObjectName, alias: Option<&TableAlias>) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(ident) = object_name_ident(name) {
        names.push(ident_key(ident));
    }
    if let Some(alias) = alias {
        names.push(ident_key(&alias.name));
    }
    names
}

fn projected(
    select: &Select,
    base_quals: &[String],
    primary_key: &[String],
    bare_columns: bool,
) -> Option<Vec<String>> {
    let mut saw_base_star = false;
    let mut columns = Vec::new();
    for item in &select.projection {
        match item {
            SelectItem::Wildcard(options) if plain_star(options) => saw_base_star = true,
            SelectItem::Wildcard(_) => return None,
            SelectItem::QualifiedWildcard(
                SelectItemQualifiedWildcardKind::ObjectName(name),
                options,
            ) if plain_star(options) => {
                saw_base_star |= object_name_ident(name)
                    .is_some_and(|ident| base_quals.iter().any(|qual| qual == &ident_key(ident)));
            }
            SelectItem::QualifiedWildcard(_, _) => return None,
            _ if saw_base_star => {}
            SelectItem::UnnamedExpr(expr) => {
                let physical = base_column(expr, base_quals, bare_columns)?;
                columns.push((physical.clone(), physical));
            }
            SelectItem::ExprWithAlias { expr, alias } => {
                columns.push((
                    base_column(expr, base_quals, bare_columns)?,
                    ident_key(alias),
                ));
            }
            _ => return None,
        }
    }
    if saw_base_star {
        return Some(primary_key.to_vec());
    }
    primary_key
        .iter()
        .map(|column| {
            columns
                .iter()
                .find(|(physical, _)| physical.eq_ignore_ascii_case(column))
                .map(|(_, visible)| visible.clone())
        })
        .collect()
}

fn base_column(expr: &Expr, quals: &[String], bare: bool) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) if bare => Some(ident_key(ident)),
        Expr::CompoundIdentifier(_) => qualified_column(expr, quals),
        _ => None,
    }
}

pub(super) fn qualified_column(expr: &Expr, quals: &[String]) -> Option<String> {
    let Expr::CompoundIdentifier(parts) = unwrap_expr(expr) else {
        return None;
    };
    if parts.len() < 2 {
        return None;
    }
    let column = ident_key(parts.last()?);
    let qualifier = ident_key(&parts[parts.len() - 2]);
    quals
        .iter()
        .any(|name| name == &qualifier)
        .then_some(column)
}

fn plain_star(options: &WildcardAdditionalOptions) -> bool {
    options.opt_ilike.is_none()
        && options.opt_exclude.is_none()
        && options.opt_except.is_none()
        && options.opt_replace.is_none()
        && options.opt_rename.is_none()
        && options.opt_alias.is_none()
}
