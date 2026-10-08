//! Parse source-owned assignment targets without discarding nested index expressions.
use sqlparser::{
    ast::{Assignment, AssignmentTarget, Expr, ObjectNamePart},
    parser::{Parser, ParserError},
    tokenizer::{Span, Token},
};

pub(in crate::codebase::postgres::source) struct Target {
    pub base: Expr,
    pub subscripts: Vec<super::expressions::Located>,
    pub indirection: Vec<Step>,
    pub span: Span,
}

pub(in crate::codebase::postgres::source) struct Facts {
    pub target: Option<Target>,
    pub value_span: Span,
    pub delimiters: Vec<Span>,
    pub span: Span,
}

pub(in crate::codebase::postgres::source) enum Step {
    Subscript { index: usize, span: Span },
    Field(sqlparser::ast::Ident),
}

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<(Assignment, Facts), ParserError> {
    let start = parser.peek_token().span.start;
    let mut assignment_target = parser.parse_assignment_target()?;
    let target = if let AssignmentTarget::ColumnName(column) = &assignment_target {
        let mut subscripts = Vec::new();
        let parts = column
            .0
            .iter()
            .filter_map(ObjectNamePart::as_ident)
            .cloned()
            .collect::<Vec<_>>();
        let mut indirection = parts
            .iter()
            .skip(1)
            .cloned()
            .map(Step::Field)
            .collect::<Vec<_>>();
        loop {
            if parser.peek_token().token == Token::LBracket {
                let start = parser.next_token().span.start;
                let expression = super::expressions::parse(parser)?;
                parser.expect_token(&Token::RBracket)?;
                let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
                indirection.push(Step::Subscript {
                    index: subscripts.len(),
                    span: Span { start, end },
                });
                subscripts.push(expression);
            } else if parser.consume_token(&Token::Period) {
                indirection.push(Step::Field(parser.parse_identifier()?));
            } else {
                break;
            }
        }
        let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
        if subscripts.is_empty() {
            None
        } else {
            let base = Expr::Identifier(parts[0].clone());
            Some(Target {
                base,
                subscripts,
                indirection,
                span: Span { start, end },
            })
        }
    } else {
        // Retain tuple grammar and existing projection completeness semantics.
        None
    };
    if let Some(Target {
        base: Expr::Identifier(base),
        ..
    }) = &target
    {
        assignment_target = AssignmentTarget::ColumnName(sqlparser::ast::ObjectName(vec![
            ObjectNamePart::Identifier(base.clone()),
        ]));
    }
    parser.expect_token(&Token::Eq)?;
    let value = super::expressions::parse(parser)?;
    let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
    Ok((
        Assignment {
            target: assignment_target,
            value: value.expression,
        },
        Facts {
            target,
            value_span: value.span,
            delimiters: value.delimiters,
            span: Span { start, end },
        },
    ))
}
