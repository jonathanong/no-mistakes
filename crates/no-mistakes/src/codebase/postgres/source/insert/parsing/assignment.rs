//! Parse source-owned assignment targets without discarding nested index expressions.
use sqlparser::{
    ast::{Assignment, AssignmentTarget, Expr, ObjectNamePart},
    parser::{Parser, ParserError},
    tokenizer::{Span, Token},
};

pub(in crate::codebase::postgres::source) struct Target {
    pub base: Expr,
    pub subscripts: Vec<(Expr, Span)>,
    pub span: Span,
}

pub(in crate::codebase::postgres::source) struct Facts {
    pub target: Option<Target>,
    pub value_span: Span,
    pub span: Span,
}

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<(Assignment, Facts), ParserError> {
    let start = parser.peek_token().span.start;
    let assignment_target = parser.parse_assignment_target()?;
    let target = if let AssignmentTarget::ColumnName(column) = &assignment_target {
        let mut subscripts = Vec::new();
        while parser.consume_token(&Token::LBracket) {
            let start = parser.peek_token().span.start;
            let index = parser.parse_expr()?;
            let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
            parser.expect_token(&Token::RBracket)?;
            subscripts.push((index, Span { start, end }));
        }
        let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
        if subscripts.is_empty() {
            None
        } else {
            let parts = column
                .0
                .iter()
                .filter_map(ObjectNamePart::as_ident)
                .cloned()
                .collect::<Vec<_>>();
            let base = if parts.len() == 1 {
                Expr::Identifier(parts[0].clone())
            } else {
                Expr::CompoundIdentifier(parts)
            };
            Some(Target {
                base,
                subscripts,
                span: Span { start, end },
            })
        }
    } else {
        // Retain tuple grammar and existing projection completeness semantics.
        None
    };
    parser.expect_token(&Token::Eq)?;
    let value_start = parser.peek_token().span.start;
    let value = parser.parse_expr()?;
    let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
    Ok((
        Assignment {
            target: assignment_target,
            value,
        },
        Facts {
            target,
            value_span: Span {
                start: value_start,
                end,
            },
            span: Span { start, end },
        },
    ))
}
