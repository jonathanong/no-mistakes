use super::*;
use oxc_ast_visit::{walk, Visit};

pub(super) fn isolated(expression: &Expression<'_>) -> bool {
    let (params, returned) = match expression {
        Expression::ArrowFunctionExpression(function) => (
            &function.params,
            function
                .body
                .as_expression()
                .or_else(|| function.body.as_function_body().and_then(direct_return)),
        ),
        Expression::FunctionExpression(function)
            if !function.generator && function.body.is_some() =>
        {
            (
                &function.params,
                function.body.as_deref().and_then(direct_return),
            )
        }
        _ => return false,
    };
    let Some(Expression::ObjectExpression(object)) = returned.map(Expression::get_inner_expression)
    else {
        return false;
    };
    if !params.items.is_empty() || params.rest.is_some() || !literal_object(object) {
        return false;
    }
    let mut execution = FactoryExecution(false);
    execution.visit_expression(expression);
    !execution.0
}

pub(super) fn actual_loader(call: &CallExpression<'_>) -> bool {
    let callee = call.callee.get_inner_expression();
    let name = match callee {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => callee
            .as_member_expression()
            .and_then(|member| member.static_property_name())
            .and_then(|name| name.as_str()),
    };
    matches!(
        name,
        Some("importActual" | "requireActual" | "importOriginal" | "require" | "eval" | "Function")
    )
}

struct FactoryExecution(bool);

impl<'a> Visit<'a> for FactoryExecution {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if actual_loader(call) {
            self.0 = true;
        } else {
            walk::walk_call_expression(self, call);
        }
    }

    fn visit_new_expression(&mut self, new: &NewExpression<'a>) {
        if matches!(new.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "Function")
        {
            self.0 = true;
        } else {
            walk::walk_new_expression(self, new);
        }
    }

    fn visit_import_expression(&mut self, _: &ImportExpression<'a>) {
        self.0 = true;
    }

    fn visit_spread_element(&mut self, _: &SpreadElement<'a>) {
        self.0 = true;
    }
}

fn direct_return<'a>(body: &'a FunctionBody<'a>) -> Option<&'a Expression<'a>> {
    let (last, preceding) = body.statements.split_last()?;
    if !preceding.iter().all(|statement| {
        matches!(
            statement,
            Statement::ExpressionStatement(_) | Statement::VariableDeclaration(_)
        )
    }) {
        return None;
    }
    let Statement::ReturnStatement(statement) = last else {
        return None;
    };
    statement.argument.as_ref()
}

fn literal_object(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().all(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        property.kind == PropertyKind::Init
            && !property.method
            && !property.computed
            && crate::ast::property_key_name(&property.key).is_some_and(|key| key != "__proto__")
            && literal_value(&property.value)
    })
}

fn literal_value(value: &Expression<'_>) -> bool {
    match value.get_inner_expression() {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::ObjectExpression(object) => literal_object(object),
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .all(|element| element.as_expression().is_some_and(literal_value)),
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        other => other.is_literal(),
    }
}
