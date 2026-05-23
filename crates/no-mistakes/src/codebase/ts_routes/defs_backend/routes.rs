fn extract_route_from_chain(expr: &Expression, register_object: &str) -> Option<String> {
    if let Expression::CallExpression(call) = expr {
        if let Some(member) = call.callee.as_member_expression() {
            let prop = member.static_property_name().unwrap_or("");
            if prop == "route" {
                if let Expression::Identifier(ident) = member.object() {
                    if ident.name.as_str() == register_object {
                        return route_arg(call.arguments.first());
                    }
                }
            } else {
                return extract_route_from_chain(member.object(), register_object);
            }
        }
    }
    None
}

fn direct_route_arg(
    call: &oxc::ast::ast::CallExpression,
    callee_object: &Expression,
    register_object: &str,
) -> Option<String> {
    let Expression::Identifier(ident) = callee_object else {
        return None;
    };
    if ident.name.as_str() != register_object {
        return None;
    }
    route_arg(call.arguments.first())
}

fn route_arg(arg: Option<&Argument>) -> Option<String> {
    match arg? {
        Argument::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Argument::TemplateLiteral(tpl) if tpl.expressions.is_empty() => {
            Some(static_template_literal(tpl))
        }
        _ => None,
    }
}

fn static_template_literal(tpl: &TemplateLiteral) -> String {
    tpl.quasis
        .iter()
        .filter_map(|quasi| quasi.value.cooked.as_deref())
        .collect::<Vec<_>>()
        .join("")
}

