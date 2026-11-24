use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Expression, ImportDeclarationSpecifier, LogicalOperator, ObjectExpression, ObjectProperty,
    ObjectPropertyKind, Program, PropertyKey, PropertyKind, Statement,
};
use oxc_span::Span;

pub(in super::super) fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(lit) => lit.value.as_str(),
        _ => None,
    }
}

pub(in super::super) fn find_property<'a>(
    obj: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a ObjectProperty<'a>> {
    obj.properties.iter().rev().find_map(|prop| {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            return None;
        };
        (prop.kind == PropertyKind::Init && property_name(&prop.key) == Some(name))
            .then_some(prop.as_ref())
    })
}

pub(in super::super) fn string_literal<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(lit) => lit.value.as_str(),
        Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => {
            tpl.quasis.first().map(|q| q.value.raw.as_str())
        }
        _ => None,
    }
}

pub(in super::super) fn non_negative_int(expr: &Expression<'_>) -> Option<(u64, Span)> {
    let Expression::NumericLiteral(lit) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    let val = lit.value;
    (val.is_finite() && val >= 0.0 && val.fract() == 0.0 && val <= u64::MAX as f64)
        .then_some((val as u64, lit.span))
}

pub(in super::super) fn bool_literal(expr: &Expression<'_>) -> Option<(bool, Span)> {
    match unwrap_ts_wrappers(expr) {
        Expression::BooleanLiteral(lit) => Some((lit.value, lit.span)),
        _ => None,
    }
}

fn static_truthiness(expr: &Expression<'_>) -> Option<bool> {
    match unwrap_ts_wrappers(expr) {
        Expression::BooleanLiteral(lit) => Some(lit.value),
        Expression::NumericLiteral(lit) => Some(lit.value != 0.0 && !lit.value.is_nan()),
        Expression::StringLiteral(lit) => Some(!lit.value.is_empty()),
        Expression::NullLiteral(_) => Some(false),
        Expression::Identifier(id) if id.name == "undefined" => Some(false),
        _ => None,
    }
}

fn static_nullish(expr: &Expression<'_>) -> Option<bool> {
    match unwrap_ts_wrappers(expr) {
        Expression::NullLiteral(_) => Some(true),
        Expression::Identifier(id) if id.name == "undefined" => Some(true),
        Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_) => Some(false),
        _ => None,
    }
}

pub(in super::super) fn collect_branch_exprs<'a>(
    expr: &'a Expression<'a>,
    out: &mut Vec<&'a Expression<'a>>,
) {
    match unwrap_ts_wrappers(expr) {
        Expression::ConditionalExpression(cond) => {
            collect_branch_exprs(&cond.consequent, out);
            collect_branch_exprs(&cond.alternate, out);
        }
        Expression::LogicalExpression(logical) => match logical.operator {
            LogicalOperator::Or => match static_truthiness(&logical.left) {
                Some(true) => collect_branch_exprs(&logical.left, out),
                Some(false) => collect_branch_exprs(&logical.right, out),
                None => {
                    collect_branch_exprs(&logical.left, out);
                    collect_branch_exprs(&logical.right, out);
                }
            },
            LogicalOperator::And => match static_truthiness(&logical.left) {
                Some(false) => collect_branch_exprs(&logical.left, out),
                Some(true) => collect_branch_exprs(&logical.right, out),
                None => {
                    collect_branch_exprs(&logical.left, out);
                    collect_branch_exprs(&logical.right, out);
                }
            },
            LogicalOperator::Coalesce => match static_nullish(&logical.left) {
                Some(true) => collect_branch_exprs(&logical.right, out),
                Some(false) => collect_branch_exprs(&logical.left, out),
                None => {
                    collect_branch_exprs(&logical.left, out);
                    collect_branch_exprs(&logical.right, out);
                }
            },
        },
        other => out.push(other),
    }
}

pub(in super::super) fn member_chain<'a>(expr: &'a Expression<'a>) -> Option<Vec<&'a str>> {
    let mut parts = Vec::new();
    let mut current = unwrap_ts_wrappers(expr);
    loop {
        match current {
            Expression::Identifier(id) => {
                parts.push(id.name.as_str());
                parts.reverse();
                return Some(parts);
            }
            Expression::StaticMemberExpression(member) if !member.optional => {
                parts.push(member.property.name.as_str());
                current = unwrap_ts_wrappers(&member.object);
            }
            _ => return None,
        }
    }
}

pub(in super::super) fn array_objects<'a>(
    expr: &'a Expression<'a>,
) -> Vec<&'a ObjectExpression<'a>> {
    let Expression::ArrayExpression(array) = unwrap_ts_wrappers(expr) else {
        return Vec::new();
    };
    array
        .elements
        .iter()
        .filter_map(|elem| match unwrap_ts_wrappers(elem.as_expression()?) {
            Expression::ObjectExpression(obj) => Some(obj.as_ref()),
            _ => None,
        })
        .collect()
}

pub(in super::super) fn project_name<'a>(obj: &'a ObjectExpression<'a>) -> Option<&'a str> {
    if let Some(name) = find_property(obj, "name").and_then(|prop| string_literal(&prop.value)) {
        return Some(name);
    }
    let test_obj =
        find_property(obj, "test").and_then(|prop| match unwrap_ts_wrappers(&prop.value) {
            Expression::ObjectExpression(inner) => Some(inner.as_ref()),
            _ => None,
        })?;
    find_property(test_obj, "name").and_then(|prop| string_literal(&prop.value))
}

pub(in super::super) fn playwright_test_bindings<'a>(program: &'a Program<'a>) -> Vec<&'a str> {
    let mut pw_bindings = Vec::new();
    let mut non_pw_test = false;
    for stmt in &program.body {
        let Statement::ImportDeclaration(decl) = stmt else {
            continue;
        };
        let src = decl.source.value.as_str().unwrap_or("");
        let is_pw = src == "@playwright/test" || src.starts_with("@playwright/experimental-ct-");
        let is_other_runner = src == "vitest" || src == "@jest/globals";
        let Some(specs) = &decl.specifiers else {
            continue;
        };
        for spec in specs {
            if let ImportDeclarationSpecifier::ImportSpecifier(named) = spec {
                let imported = named.imported.name().as_str();
                let local = named.local.name.as_str();
                if is_pw && imported == "test" {
                    pw_bindings.push(local);
                } else if is_other_runner && local == "test" {
                    non_pw_test = true;
                }
            }
        }
    }
    if !pw_bindings.is_empty() {
        return pw_bindings;
    }
    if non_pw_test {
        Vec::new()
    } else {
        vec!["test"]
    }
}
