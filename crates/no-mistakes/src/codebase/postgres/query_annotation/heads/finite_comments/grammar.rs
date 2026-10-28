use super::{Domain, Root, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, ForOfStatement, ForStatementLeft};

pub(in crate::codebase::postgres::query_annotation::heads) fn loop_domain(
    value: &ForOfStatement<'_>,
) -> Option<Domain> {
    if value.r#await {
        return None;
    }
    let ForStatementLeft::VariableDeclaration(left) = &value.left else {
        return None;
    };
    if !left.kind.is_const() {
        return None;
    }
    let [declaration] = left.declarations.as_slice() else {
        return None;
    };
    let BindingPattern::ArrayPattern(pattern) = &declaration.id else {
        return None;
    };
    let [Some(BindingPattern::BindingIdentifier(first)), Some(BindingPattern::BindingIdentifier(_))] =
        pattern.elements.as_slice()
    else {
        return None;
    };
    if pattern.rest.is_some() {
        return None;
    }
    let Expression::ArrayExpression(rows) = unwrap_ts_wrappers(&value.right) else {
        return None;
    };
    if rows.elements.is_empty() || rows.elements.len() > 16 {
        return None;
    }
    let mut values = Vec::new();
    for row in &rows.elements {
        let Expression::ArrayExpression(row) = row.as_expression()? else {
            return None;
        };
        let [first, second] = row.elements.as_slice() else {
            return None;
        };
        let Expression::StringLiteral(value) = first.as_expression()? else {
            return None;
        };
        if !matches!(second.as_expression()?, Expression::StringLiteral(_)) {
            return None;
        }
        let name = crate::ast::js_literal(value).into_owned();
        if name.trim().is_empty() {
            return None;
        }
        if !values.contains(&name) {
            values.push(name);
        }
    }
    Some(Domain {
        declaration: (first.span.start, first.span.end),
        name: first.name.to_string(),
        values,
    })
}

pub(in crate::codebase::postgres::query_annotation::heads) fn template(
    expression: &Expression<'_>,
) -> Option<(Span, String, String)> {
    let Expression::TemplateLiteral(value) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    let Expression::Identifier(reference) = value.expressions.first()? else {
        return None;
    };
    let before = value
        .quasis
        .first()
        .filter(|part| part.value.cooked.is_some())
        .map(crate::ast::template_quasi_text)?
        .into_owned();
    let after = value
        .quasis
        .get(1)
        .filter(|part| part.value.cooked.is_some())
        .map(crate::ast::template_quasi_text)?
        .into_owned();
    if !before.trim_start().starts_with("/*") || before.contains("*/") || !after.contains("*/") {
        return None;
    }
    let end = after.find("*/")? + 2;
    Some((
        (reference.span.start, reference.span.end),
        before,
        after[..end].to_string(),
    ))
}

pub(in crate::codebase::postgres::query_annotation::heads) fn root(
    declaration: Span,
    function: Span,
    init: &Expression<'_>,
) -> Option<Root> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(init) else {
        return None;
    };
    let receiver = super::super::append::prefix_receiver(call)?;
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(receiver) else {
        return None;
    };
    if !tagged.quasi.expressions.is_empty()
        || tagged.quasi.quasis.len() != 1
        || !tagged.quasi.quasis[0]
            .value
            .cooked
            .as_ref()
            .is_some_and(|text| text.is_empty())
    {
        return None;
    }
    let Expression::Identifier(tag) = unwrap_ts_wrappers(&tagged.tag) else {
        return None;
    };
    let [argument] = call.arguments.as_slice() else {
        return None;
    };
    let (reference, before, after) = template(argument.as_expression()?)?;
    Some(Root {
        declaration,
        function,
        reference,
        tag: (tag.span.start, tag.span.end),
        before,
        after,
    })
}
