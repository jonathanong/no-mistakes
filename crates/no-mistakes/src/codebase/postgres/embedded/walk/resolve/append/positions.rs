use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::{
    placeholders, source_positions, EmbeddedSqlSourcePosition,
};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{CallExpression, Expression};

pub(in crate::codebase::postgres::embedded::walk) fn fragment(
    call: &CallExpression<'_>,
    visitor: &ScopeVisitor<'_>,
    offset: u32,
) -> (u32, Vec<EmbeddedSqlSourcePosition>) {
    // apply_append has already verified that its first argument is static.
    let expression = call.arguments[0].as_expression().unwrap();
    if let Expression::Identifier(ident) = unwrap_ts_wrappers(expression) {
        let binding = visitor.lookup(ident.name.as_str()).unwrap();
        let mut positions = binding.sql_source_positions;
        renumber(&mut positions, binding.sql.as_deref().unwrap(), offset);
        return (binding.line, positions);
    }
    let line = byte_offset_to_line(visitor.source, call.span.start as usize);
    let positions = source_positions::for_expression_with_offset(
        expression,
        visitor.source,
        call.span.start as usize,
        line,
        offset as usize,
    );
    (line, positions)
}

pub(in crate::codebase::postgres::embedded) fn renumber(
    positions: &mut [EmbeddedSqlSourcePosition],
    sql: &str,
    offset: u32,
) {
    for position in positions {
        let original_column = position.sql_column;
        for (index, (at, _)) in sql
            .match_indices(placeholders::INTERNAL_PLACEHOLDER)
            .enumerate()
        {
            let digits = sql[at + placeholders::INTERNAL_PLACEHOLDER.len()..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            let delta = (offset as usize + index + 1).to_string().len() as i32 - digits as i32;
            let prefix = placeholders::publish_placeholders(sql[..at].to_string());
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1;
            let column = prefix.rsplit('\n').next().unwrap().chars().count() as u32 + 1;
            if position.sql_line == line && original_column > column {
                position.sql_column = position.sql_column.saturating_add_signed(delta);
            }
        }
    }
}

pub(in crate::codebase::postgres::embedded) fn append(
    positions: &mut Vec<EmbeddedSqlSourcePosition>,
    prefix: &str,
    origin: u32,
    fragment_origin: u32,
    fragment_positions: &[EmbeddedSqlSourcePosition],
) {
    let prefix = placeholders::publish_placeholders(prefix.to_string());
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1;
    let column = prefix.rsplit('\n').next().unwrap().chars().count() as u32 + 1;
    let expected = positions
        .last()
        .map(|position| position.source_line + line - position.sql_line)
        .unwrap_or(origin + line - 1);
    if expected != fragment_origin {
        positions.push(EmbeddedSqlSourcePosition {
            sql_line: line,
            sql_column: column,
            source_line: fragment_origin,
        });
    }
    positions.extend(
        fragment_positions
            .iter()
            .map(|position| EmbeddedSqlSourcePosition {
                sql_line: line + position.sql_line - 1,
                sql_column: position.sql_column
                    + if position.sql_line == 1 {
                        column - 1
                    } else {
                        0
                    },
                source_line: position.source_line,
            }),
    );
}
