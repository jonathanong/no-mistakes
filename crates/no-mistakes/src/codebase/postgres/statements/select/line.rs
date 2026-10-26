use sqlparser::ast::{Select, Spanned, TableFactor, TableWithJoins};

pub(super) fn select_line(select: &Select, tables: &[String]) -> usize {
    // Preserve relation-line directives, but never search another statement's text.
    tables
        .first()
        .and_then(|name| relation_line(&select.from, name))
        .unwrap_or(select.select_token.0.span.start.line.max(1) as usize)
}

fn relation_line(from: &[TableWithJoins], target: &str) -> Option<usize> {
    from.iter().find_map(|table| {
        factor_line(&table.relation, target).or_else(|| {
            table
                .joins
                .iter()
                .find_map(|join| factor_line(&join.relation, target))
        })
    })
}

fn factor_line(factor: &TableFactor, target: &str) -> Option<usize> {
    match factor {
        TableFactor::Table { name, .. }
            if crate::codebase::postgres::idents::object_name_key(name) == target =>
        {
            Some(name.span().start.line.max(1) as usize)
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => relation_line(std::slice::from_ref(table_with_joins), target),
        _ => None,
    }
}
