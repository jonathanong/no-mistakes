use super::ScopeVisitor;
use crate::codebase::postgres::EmbeddedSqlSourcePosition;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(super) fn fill(
    visitor: &ScopeVisitor<'_>,
    argument: &Expression<'_>,
    sql: Option<&str>,
    mapping: (u32, &[EmbeddedSqlSourcePosition], &mut Vec<u32>),
) {
    let (line, positions, offsets) = mapping;
    if !visitor.exact_sql_origins
        || !offsets.is_empty()
        || positions
            .iter()
            .any(|position| position.source_path.is_some())
    {
        return;
    }
    let Some(sql) = sql else { return };
    if !matches!(
        unwrap_ts_wrappers(argument),
        Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
    ) {
        return;
    }
    let canonical = super::super::super::unpublished_sql_text(argument);
    let origins = if canonical.as_deref() == Some(sql) {
        Some(super::super::super::source_positions::origins::expression(
            argument,
            visitor.source,
            sql,
        ))
    } else {
        matching_origins(visitor, argument, sql)
            .filter(|origins| compatible(visitor.source, sql, origins, line, positions))
    };
    if let Some(origins) = origins.filter(|origins| origins.len() == sql.len()) {
        *offsets = super::rewrite_origins(sql, &origins, 0, true);
    }
}

fn matching_origins(
    visitor: &ScopeVisitor<'_>,
    argument: &Expression<'_>,
    sql: &str,
) -> Option<Vec<u32>> {
    if let Some(origins) = recovered_origins(visitor, argument, sql) {
        return Some(origins);
    }
    let Expression::TemplateLiteral(template) = unwrap_ts_wrappers(argument) else {
        return None;
    };
    let mut text = String::new();
    let mut origins = Vec::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        let decoded = super::super::super::tags::quasi_text(quasi, false);
        text.push_str(&decoded);
        let raw = &visitor.source[quasi.span.start as usize..quasi.span.end as usize];
        origins.extend(super::super::super::source_positions::origins::piece(
            raw,
            &decoded,
            quasi.span.start,
            quasi.value.cooked.is_none(),
        ));
        if let Some(hole) = template.expressions.get(index) {
            let value = super::compose::static_fragment(hole, visitor)?;
            origins.extend(recovered_origins(visitor, hole, &value)?);
            text.push_str(&value);
        }
    }
    (text == sql && origins.len() == sql.len()).then_some(origins)
}

fn recovered_origins(
    visitor: &ScopeVisitor<'_>,
    argument: &Expression<'_>,
    sql: &str,
) -> Option<Vec<u32>> {
    let mut matching = visitor
        .recover_variants(argument)?
        .into_iter()
        .filter(|value| value.sql == sql);
    let first = matching.next()?;
    let verified = |value: &super::super::variants::Recovered| {
        value.origins.len() == sql.len()
            && value
                .positions
                .iter()
                .all(|position| position.source_path.is_none())
            && authored_characters(visitor.source, sql, &value.origins)
    };
    if !verified(&first)
        || !matching.all(|value| verified(&value) && value.origins == first.origins)
    {
        return None;
    }
    Some(first.origins)
}

fn authored_characters(source: &str, sql: &str, origins: &[u32]) -> bool {
    sql.char_indices().all(|(offset, character)| {
        origins
            .get(offset)
            .and_then(|origin| source.get(*origin as usize..))
            .and_then(|text| text.chars().next())
            == Some(character)
    })
}

fn compatible(
    source: &str,
    sql: &str,
    origins: &[u32],
    base: u32,
    positions: &[EmbeddedSqlSourcePosition],
) -> bool {
    if origins.len() != sql.len() {
        return false;
    }
    let newlines: Vec<_> = source
        .match_indices('\n')
        .map(|(offset, _)| offset)
        .collect();
    let (mut line, mut column) = (1u32, 1u32);
    for (offset, character) in sql.char_indices() {
        let expected = positions
            .iter()
            .rev()
            .find(|position| (position.sql_line, position.sql_column) <= (line, column))
            .map_or(base + line - 1, |position| {
                position.source_line + line - position.sql_line
            });
        let actual =
            1 + newlines.partition_point(|newline| *newline < origins[offset] as usize) as u32;
        if actual != expected {
            return false;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    true
}

#[cfg(test)]
mod tests;
