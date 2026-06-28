fn collect_query_params_from_expression(
    expr: &Expression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    match expr {
        Expression::CallExpression(call) => {
            collect_query_params_from_call_expression(call, params, named_handlers);
        }
        Expression::StaticMemberExpression(member) => {
            collect_query_params_from_static_member_expression(member, params, named_handlers);
        }
        Expression::ComputedMemberExpression(member) => {
            collect_query_params_from_computed_member_expression(member, params, named_handlers);
        }
        Expression::ChainExpression(chain) => {
            collect_query_params_from_chain_element(&chain.expression, params, named_handlers);
        }
        Expression::AssignmentExpression(assign) => {
            collect_query_params_from_expression(&assign.right, params, named_handlers);
        }
        Expression::ConditionalExpression(expr) => {
            collect_query_params_from_expression(&expr.test, params, named_handlers);
            collect_query_params_from_expression(&expr.consequent, params, named_handlers);
            collect_query_params_from_expression(&expr.alternate, params, named_handlers);
        }
        Expression::LogicalExpression(expr) => {
            collect_query_params_from_expression(&expr.left, params, named_handlers);
            collect_query_params_from_expression(&expr.right, params, named_handlers);
        }
        Expression::BinaryExpression(expr) => {
            collect_query_params_from_expression(&expr.left, params, named_handlers);
            collect_query_params_from_expression(&expr.right, params, named_handlers);
        }
        Expression::SequenceExpression(expr) => {
            for expression in &expr.expressions {
                collect_query_params_from_expression(expression, params, named_handlers);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property {
                    collect_query_params_from_expression(&property.value, params, named_handlers);
                }
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
            {
                collect_query_params_from_expression(element, params, named_handlers);
            }
        }
        Expression::AwaitExpression(expr) => {
            collect_query_params_from_expression(&expr.argument, params, named_handlers)
        }
        Expression::ParenthesizedExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers);
        }
        Expression::TSAsExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers)
        }
        Expression::TSTypeAssertion(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers);
        }
        Expression::TSNonNullExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers);
        }
        Expression::TSSatisfiesExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers);
        }
        _ => {}
    }
}

fn collect_query_params_from_chain_element(
    chain: &ChainElement<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    match chain {
        ChainElement::CallExpression(call) => {
            collect_query_params_from_call_expression(call, params, named_handlers);
        }
        other => {
            if let Some(member) = other.as_member_expression() {
                collect_query_params_from_member_expression(member, params, named_handlers);
            }
        }
    }
}

fn collect_query_params_from_member_expression(
    member: &oxc_ast::ast::MemberExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    if let Some(property) = member.static_property_name() {
        if expression_is_query_object(member.object()) && property != "query" {
            params.insert(property.to_string());
        }
    }
    collect_query_params_from_expression(member.object(), params, named_handlers);
}

fn collect_query_params_from_call_expression(
    call: &oxc_ast::ast::CallExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    if let Some(name) = query_param_from_call(call) {
        params.insert(name);
    }
    if let Expression::Identifier(id) = &call.callee {
        if let Some(handler_params) = named_handlers.get(id.name.as_str()) {
            params.extend(handler_params.iter().cloned());
        }
    }
    collect_query_params_from_expression(&call.callee, params, named_handlers);
    for arg in &call.arguments {
        if let Some(expr) = arg.as_expression() {
            collect_query_params_from_expression(expr, params, named_handlers);
        }
    }
}

fn collect_query_params_from_static_member_expression(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    if expression_is_query_object(&member.object) {
        params.insert(member.property.name.as_str().to_string());
    }
    collect_query_params_from_expression(&member.object, params, named_handlers);
}

fn collect_query_params_from_computed_member_expression(
    member: &oxc_ast::ast::ComputedMemberExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    if let Some(name) = computed_query_param_name(member) {
        params.insert(name);
    }
    collect_query_params_from_expression(&member.object, params, named_handlers);
    collect_query_params_from_expression(&member.expression, params, named_handlers);
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
