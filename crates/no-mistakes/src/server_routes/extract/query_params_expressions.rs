fn collect_query_params_from_expression(
    expr: &Expression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    match expr {
        Expression::CallExpression(call) => {
            collect_query_params_from_call_expression(call, params, named_handlers, state);
        }
        Expression::StaticMemberExpression(member) => {
            collect_query_params_from_static_member_expression(member, params, named_handlers, state);
        }
        Expression::ComputedMemberExpression(member) => {
            collect_query_params_from_computed_member_expression(member, params, named_handlers, state);
        }
        Expression::ChainExpression(chain) => {
            collect_query_params_from_chain_element(&chain.expression, params, named_handlers, state);
        }
        Expression::AssignmentExpression(assign) => {
            collect_query_params_from_expression(&assign.right, params, named_handlers, state);
        }
        Expression::ConditionalExpression(expr) => {
            collect_query_params_from_expression(&expr.test, params, named_handlers, state);
            collect_query_params_from_expression(&expr.consequent, params, named_handlers, state);
            collect_query_params_from_expression(&expr.alternate, params, named_handlers, state);
        }
        Expression::LogicalExpression(expr) => {
            collect_query_params_from_expression(&expr.left, params, named_handlers, state);
            collect_query_params_from_expression(&expr.right, params, named_handlers, state);
        }
        Expression::BinaryExpression(expr) => {
            collect_query_params_from_expression(&expr.left, params, named_handlers, state);
            collect_query_params_from_expression(&expr.right, params, named_handlers, state);
        }
        Expression::SequenceExpression(expr) => {
            for expression in &expr.expressions {
                collect_query_params_from_expression(expression, params, named_handlers, state);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property {
                    collect_query_params_from_expression(&property.value, params, named_handlers, state);
                }
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
            {
                collect_query_params_from_expression(element, params, named_handlers, state);
            }
        }
        Expression::AwaitExpression(expr) => {
            collect_query_params_from_expression(&expr.argument, params, named_handlers, state)
        }
        Expression::ParenthesizedExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers, state);
        }
        Expression::TSAsExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers, state)
        }
        Expression::TSTypeAssertion(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers, state);
        }
        Expression::TSNonNullExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers, state);
        }
        Expression::TSSatisfiesExpression(expr) => {
            collect_query_params_from_expression(&expr.expression, params, named_handlers, state);
        }
        _ => {}
    }
}

fn collect_query_params_from_chain_element(
    chain: &ChainElement<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    match chain {
        ChainElement::CallExpression(call) => {
            collect_query_params_from_call_expression(call, params, named_handlers, state);
        }
        other => {
            if let Some(member) = other.as_member_expression() {
                collect_query_params_from_member_expression(member, params, named_handlers, state);
            }
        }
    }
}

fn collect_query_params_from_member_expression(
    member: &oxc_ast::ast::MemberExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    if let Some(property) = member.static_property_name() {
        if expression_is_query_object(member.object(), &state.query_aliases) && property != "query" {
            params.insert(property.to_string());
        }
    }
    collect_query_params_from_expression(member.object(), params, named_handlers, state);
}

fn collect_query_params_from_call_expression(
    call: &oxc_ast::ast::CallExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    if let Some(name) = query_param_from_call(call) {
        params.insert(name);
    }
    if let Expression::Identifier(id) = &call.callee {
        if let Some(handler_params) = named_handlers.get(id.name.as_str()) {
            params.extend(handler_params.iter().cloned());
        }
    }
    collect_query_params_from_expression(&call.callee, params, named_handlers, state);
    for arg in &call.arguments {
        if let Some(expr) = arg.as_expression() {
            collect_query_params_from_expression(expr, params, named_handlers, state);
        }
    }
}

fn collect_query_params_from_static_member_expression(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    if expression_is_query_object(&member.object, &state.query_aliases) {
        params.insert(member.property.name.as_str().to_string());
    }
    collect_query_params_from_expression(&member.object, params, named_handlers, state);
}

fn collect_query_params_from_computed_member_expression(
    member: &oxc_ast::ast::ComputedMemberExpression<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    if let Some(name) = computed_query_param_name(member, &state.query_aliases) {
        params.insert(name);
    }
    collect_query_params_from_expression(&member.object, params, named_handlers, state);
    collect_query_params_from_expression(&member.expression, params, named_handlers, state);
}

fn computed_query_param_name(
    member: &oxc_ast::ast::ComputedMemberExpression<'_>,
    query_aliases: &BTreeSet<String>,
) -> Option<String> {
    if !expression_is_query_object(&member.object, query_aliases) {
        return None;
    }
    match &member.expression {
        Expression::StringLiteral(value) => Some(value.value.as_str().to_string()),
        _ => None,
    }
}
