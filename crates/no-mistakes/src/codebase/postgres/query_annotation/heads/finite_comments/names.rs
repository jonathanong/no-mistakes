//! Closed string-literal name sets: one ternary, or one `for-of` list.
use super::Domain;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, ForOfStatement, ForStatementLeft};

pub(in crate::codebase::postgres::query_annotation::heads) struct InlineName {
    pub values: Vec<String>,
    pub before: String,
    pub after: String,
}

pub(in crate::codebase::postgres::query_annotation::heads) fn string_list(
    value: &ForOfStatement<'_>,
) -> Option<Domain> {
    let ForStatementLeft::VariableDeclaration(left) = &value.left else {
        return None;
    };
    if !left.kind.is_const() {
        return None;
    }
    let [declaration] = left.declarations.as_slice() else {
        return None;
    };
    let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
        return None;
    };
    let values = string_values(unwrap_ts_wrappers(&value.right))?;
    Some(Domain {
        declaration: (binding.span.start, binding.span.end),
        name: binding.name.to_string(),
        values,
        import_gated: true,
    })
}

pub(in crate::codebase::postgres::query_annotation::heads) fn plain_ternary(
    expression: &Expression<'_>,
) -> Option<Vec<String>> {
    let Expression::ConditionalExpression(value) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    let mut values = Vec::new();
    for arm in [&value.consequent, &value.alternate] {
        let name = string_literal(arm)?;
        if name.trim().is_empty() {
            return None;
        }
        if !values.contains(&name) {
            values.push(name);
        }
    }
    Some(values)
}

pub(in crate::codebase::postgres::query_annotation::heads) fn inline_name(
    expression: &Expression<'_>,
) -> Option<InlineName> {
    let (first, before, after) = comment_bounds(expression)?;
    Some(InlineName {
        values: plain_ternary(first)?,
        before,
        after,
    })
}

pub(super) fn comment_bounds<'a>(
    expression: &'a Expression<'a>,
) -> Option<(&'a Expression<'a>, String, String)> {
    let Expression::TemplateLiteral(value) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    let first = value.expressions.first()?;
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
    Some((first, before, after[..end].to_string()))
}

fn string_values(expression: &Expression<'_>) -> Option<Vec<String>> {
    let Expression::ArrayExpression(rows) = expression else {
        return None;
    };
    if rows.elements.is_empty() || rows.elements.len() > 16 {
        return None;
    }
    let mut values = Vec::new();
    for row in &rows.elements {
        let name = string_literal(row.as_expression()?)?;
        if name.trim().is_empty() {
            return None;
        }
        if !values.contains(&name) {
            values.push(name);
        }
    }
    Some(values)
}

fn string_literal(expression: &Expression<'_>) -> Option<String> {
    let Expression::StringLiteral(value) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    Some(crate::ast::js_literal(value).into_owned())
}
