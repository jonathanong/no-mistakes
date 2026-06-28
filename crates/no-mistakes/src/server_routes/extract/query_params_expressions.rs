fn collect_query_params_from_expression(expr: &Expression<'_>, params: &mut BTreeSet<String>) {
    match expr {
        Expression::CallExpression(call) => {
            if let Some(name) = query_param_from_call(call) {
                params.insert(name);
            }
            collect_query_params_from_expression(&call.callee, params);
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    collect_query_params_from_expression(expr, params);
                }
            }
        }
        Expression::StaticMemberExpression(member) => {
            if expression_is_query_object(&member.object) {
                params.insert(member.property.name.as_str().to_string());
            }
            collect_query_params_from_expression(&member.object, params);
        }
        Expression::ComputedMemberExpression(member) => {
            if let Some(name) = computed_query_param_name(member) {
                params.insert(name);
            }
            collect_query_params_from_expression(&member.object, params);
            collect_query_params_from_expression(&member.expression, params);
        }
        Expression::AssignmentExpression(assign) => {
            collect_query_params_from_expression(&assign.right, params);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            for statement in &arrow.body.statements {
                collect_query_params_from_statement(statement, params);
            }
        }
        Expression::FunctionExpression(function) => {
            collect_query_params_from_optional_function_body(function.body.as_ref(), params);
        }
        Expression::ConditionalExpression(expr) => {
            collect_query_params_from_expression(&expr.test, params);
            collect_query_params_from_expression(&expr.consequent, params);
            collect_query_params_from_expression(&expr.alternate, params);
        }
        Expression::LogicalExpression(expr) => {
            collect_query_params_from_expression(&expr.left, params);
            collect_query_params_from_expression(&expr.right, params);
        }
        Expression::BinaryExpression(expr) => {
            collect_query_params_from_expression(&expr.left, params);
            collect_query_params_from_expression(&expr.right, params);
        }
        Expression::SequenceExpression(expr) => {
            for expression in &expr.expressions {
                collect_query_params_from_expression(expression, params);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property {
                    collect_query_params_from_expression(&property.value, params);
                }
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
            {
                collect_query_params_from_expression(element, params);
            }
        }
        Expression::AwaitExpression(expr) => {
            collect_query_params_from_expression(&expr.argument, params)
        }
        Expression::ParenthesizedExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params);
        }
        Expression::TSAsExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params)
        }
        Expression::TSTypeAssertion(expr) => {
            collect_query_params_from_expression(&expr.expression, params);
        }
        Expression::TSNonNullExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params);
        }
        Expression::TSSatisfiesExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params);
        }
        _ => {}
    }
}

fn computed_query_param_name(
    member: &oxc_ast::ast::ComputedMemberExpression<'_>,
) -> Option<String> {
    if !expression_is_query_object(&member.object) {
        return None;
    }
    match &member.expression {
        Expression::StringLiteral(value) => Some(value.value.as_str().to_string()),
        _ => None,
    }
}
