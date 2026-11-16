use super::{for_expression, EmbeddedSqlSourcePosition};
use crate::codebase::postgres::embedded::{placeholders, unpublished_sql_text};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(in crate::codebase::postgres::embedded) fn for_rendered_expression(
    expr: &Expression<'_>,
    source: &str,
    start: usize,
    line: u32,
    sql: Option<&str>,
) -> Vec<EmbeddedSqlSourcePosition> {
    let mut positions = for_expression(expr, source, start, line);
    let Some(sql) = sql.filter(|_| {
        matches!(
            unwrap_ts_wrappers(expr),
            Expression::TaggedTemplateExpression(_)
        )
    }) else {
        return positions;
    };
    let Some(canonical) = unpublished_sql_text(expr) else {
        return positions;
    };
    let binds: Vec<_> = placeholders::markers(sql)
        .map(|(at, internal, _)| {
            (
                internal == placeholders::INTERNAL_BIND,
                sql[at + internal.len()..]
                    .bytes()
                    .take_while(u8::is_ascii_digit)
                    .count(),
            )
        })
        .collect();
    if placeholders::markers(&canonical).count() != binds.len() {
        return positions;
    }
    let mut shifts = Vec::new();
    let (mut cursor, mut sql_line, mut sql_column) = (0, 1, 1);
    for ((at, internal, public), (bind, bind_digits)) in
        placeholders::markers(&canonical).zip(binds)
    {
        advance(&canonical[cursor..at], &mut sql_line, &mut sql_column);
        let digits = canonical[at + internal.len()..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if bind {
            shifts.push((
                sql_line,
                sql_column,
                (public.len() + digits - 1 - bind_digits) as u32,
            ));
        }
        sql_column += (public.len() + digits) as u32;
        cursor = at + internal.len() + digits;
    }
    for position in &mut positions {
        let shift: u32 = shifts
            .iter()
            .filter(|&&(line, column, _)| line == position.sql_line && column < position.sql_column)
            .map(|&(_, _, shift)| shift)
            .sum();
        position.sql_column = position.sql_column.saturating_sub(shift);
    }
    positions
}

fn advance(text: &str, line: &mut u32, column: &mut u32) {
    for character in text.chars() {
        if character == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
    }
}
