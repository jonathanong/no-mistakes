//! Exact literal Set construction and the dominating throwing membership test.
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, BindingPattern, Expression, Statement, UnaryOperator};

pub(super) struct LiteralSet {
    pub name: String,
    pub values: Vec<String>,
    pub constructor: u32,
    pub declaration: u32,
}

pub(super) fn literal_set(declaration: &ast::VariableDeclaration<'_>) -> Option<LiteralSet> {
    if !declaration.kind.is_const() || declaration.declarations.len() != 1 {
        return None;
    }
    let item = &declaration.declarations[0];
    let BindingPattern::BindingIdentifier(set) = &item.id else {
        return None;
    };
    let Expression::NewExpression(created) = unwrap_ts_wrappers(item.init.as_ref()?) else {
        return None;
    };
    let Expression::Identifier(constructor) = unwrap_ts_wrappers(&created.callee) else {
        return None;
    };
    if constructor.name != "Set" || created.arguments.len() != 1 {
        return None;
    }
    let Expression::ArrayExpression(array) =
        unwrap_ts_wrappers(created.arguments[0].as_expression()?)
    else {
        return None;
    };
    if array.elements.is_empty() || array.elements.len() > 16 {
        return None;
    }
    let mut unique = FxHashSet::default();
    let mut values = Vec::new();
    for element in &array.elements {
        let value = string_element(element.as_expression()?)?;
        if value.is_empty()
            || value.contains('\0')
            || crate::codebase::postgres::embedded::helpers::sentinel::contains(&value)
        {
            return None;
        }
        if unique.insert(value.clone()) {
            values.push(value);
        }
    }
    Some(LiteralSet {
        name: set.name.to_string(),
        values,
        constructor: constructor.span.start,
        declaration: set.span.start,
    })
}

fn string_element(element: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(element) {
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(value).into_owned()),
        Expression::TemplateLiteral(value) if value.expressions.is_empty() => {
            let quasi = value.quasis.first()?;
            let _ = quasi.value.cooked.as_ref()?;
            Some(crate::ast::template_quasi_text(quasi).into_owned())
        }
        _ => None,
    }
}

pub(super) struct Membership {
    pub argument: String,
    pub receiver: u32,
}

pub(super) fn throwing_membership(
    function: &ast::Function<'_>,
    guard: &ast::IfStatement<'_>,
    set: &str,
) -> Option<Membership> {
    let Expression::UnaryExpression(negation) = unwrap_ts_wrappers(&guard.test) else {
        return None;
    };
    if negation.operator != UnaryOperator::LogicalNot || guard.alternate.is_some() {
        return None;
    }
    let Expression::CallExpression(has) = unwrap_ts_wrappers(&negation.argument) else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&has.callee) else {
        return None;
    };
    let Expression::Identifier(receiver) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let Expression::Identifier(argument) =
        unwrap_ts_wrappers(has.arguments.first()?.as_expression()?)
    else {
        return None;
    };
    if has.optional
        || has.arguments.len() != 1
        || member.optional
        || member.property.name != "has"
        || receiver.name != set
        || set == argument.name
        || !throws(&guard.consequent)
        || !function.params.items.iter().any(|param| {
            param.initializer.is_none()
                && matches!(&param.pattern, BindingPattern::BindingIdentifier(id) if id.name == argument.name)
        })
    {
        return None;
    }
    Some(Membership {
        argument: argument.name.to_string(),
        receiver: receiver.span.start,
    })
}

fn throws(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => {
            matches!(block.body.as_slice(), [Statement::ThrowStatement(_)])
        }
        _ => false,
    }
}
