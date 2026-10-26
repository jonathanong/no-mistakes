fn extract_pattern_from_jsx_expression(jsx_expr: &JSXExpression) -> Option<String> {
    match jsx_expr {
        JSXExpression::EmptyExpression(_) => None,
        _ => jsx_expr
            .as_expression()
            .and_then(extract_pattern_from_expression),
    }
}

fn check_call_for_route_ref(
    call: &oxc_ast::ast::CallExpression,
    source: &str,
    file: &str,
    router_bindings: &RouterBindings<'_>,
    refs: &mut Vec<RouteRef>,
) {
    // Detect router.push('/path') / router.replace('/path') where router is
    // bound to useRouter().
    if let Some(member) = call.callee.as_member_expression() {
        let is_router_method = crate::ast::member_property_name(member)
            .is_some_and(|prop| prop == "push" || prop == "replace" || prop == "prefetch");
        if is_router_method {
            if let Expression::Identifier(ident) = member.object() {
                let name = ident.name.as_str();
                if router_bindings.objects.contains(name) {
                    let line = byte_offset_to_line(source, call.span.start as usize);
                    if let Some(pattern) =
                        first_arg_pattern(&call.arguments).filter(|p| !should_skip(p))
                    {
                        refs.push(RouteRef {
                            pattern,
                            file: file.to_string(),
                            line,
                            method: Some("GET".to_string()),
                        });
                    }
                }
            }
        }
    }

    if let Expression::Identifier(id) = &call.callee {
        let name = id.name.as_str();
        if router_bindings.redirects.contains(name) || router_bindings.methods.contains(name) {
            let line = byte_offset_to_line(source, call.span.start as usize);
            if let Some(pattern) = first_arg_pattern(&call.arguments).filter(|p| !should_skip(p)) {
                refs.push(RouteRef {
                    pattern,
                    file: file.to_string(),
                    line,
                    method: Some("GET".to_string()),
                });
            }
        }
    }

    let is_fetch = match &call.callee {
        Expression::Identifier(id) => id.name.as_str() == "fetch",
        other => other
            .as_member_expression()
            .and_then(|m| crate::ast::member_property_name(m))
            .map(|n| n == "fetch")
            .unwrap_or(false),
    };

    if is_fetch {
        let line = byte_offset_to_line(source, call.span.start as usize);
        if let Some(pattern) = first_arg_pattern(&call.arguments) {
            // Only capture fetch() calls to local absolute paths (starting with '/').
            // External URLs (http/https) are already filtered by should_skip().
            if pattern.starts_with('/') && !should_skip(&pattern) {
                refs.push(RouteRef {
                    pattern,
                    file: file.to_string(),
                    line,
                    method: fetch_method(&call.arguments),
                });
            }
        }
    }
}

fn fetch_method(arguments: &oxc_allocator::Vec<Argument>) -> Option<String> {
    let Some(Argument::ObjectExpression(options)) = arguments.get(1) else {
        return Some("GET".to_string());
    };
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if crate::ast::property_key_name(&property.key).is_some_and(|name| name == "method") {
            if let Expression::StringLiteral(value) = &property.value {
                return Some(crate::ast::js_literal(value).to_ascii_uppercase());
            }
            return None;
        }
    }
    Some("GET".to_string())
}

fn first_arg_pattern(arguments: &oxc_allocator::Vec<Argument>) -> Option<String> {
    let first = arguments.first()?;
    match first {
        Argument::StringLiteral(s) => Some(crate::ast::js_literal(&s).into_owned()),
        Argument::TemplateLiteral(tpl) => Some(normalize_template(tpl)),
        _ => {
            if let Some(expr) = first.as_expression() {
                extract_pattern_from_expression(expr)
            } else {
                None
            }
        }
    }
}

fn extract_pattern_from_expression(expr: &Expression) -> Option<String> {
    match expr {
        Expression::StringLiteral(s) => Some(normalize_next_pathname_pattern(
            crate::ast::js_literal(&s).as_ref(),
        )),
        Expression::TemplateLiteral(tpl) => Some(normalize_template(tpl)),
        Expression::ObjectExpression(obj) => object_route_pattern(obj),
        Expression::TSTypeAssertion(ts_assertion) => {
            extract_pattern_from_expression(&ts_assertion.expression)
        }
        _ => None,
    }
}

include!("patterns_route_objects.rs");

pub(crate) fn normalize_next_pathname_pattern(path: &str) -> String {
    let leading_slash = path.starts_with('/');
    let trailing_slash = path.ends_with('/') && path.len() > 1;
    let segments: Vec<String> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            if segment.starts_with("[[...") && segment.ends_with("]]") {
                "**".to_string()
            } else if segment.starts_with("[...") && segment.ends_with(']') {
                "*".to_string()
            } else if segment.starts_with('[') && segment.ends_with(']') {
                format!(":{}", &segment[1..segment.len() - 1])
            } else {
                segment.to_string()
            }
        })
        .collect();

    let mut normalized = if leading_slash {
        format!("/{}", segments.join("/"))
    } else {
        segments.join("/")
    };
    if trailing_slash {
        normalized.push('/');
    }
    normalized
}

/// Normalize a template literal to a route pattern (replaces `${...}` with `:param`).
pub fn normalize_template(tpl: &TemplateLiteral) -> String {
    let mut result = String::new();
    for (i, quasi) in tpl.quasis.iter().enumerate() {
        if quasi.value.cooked.is_some() {
            result.push_str(crate::ast::template_quasi_text(quasi).as_ref());
        }
        if i < tpl.expressions.len() {
            result.push_str(":param");
        }
    }
    normalize_next_pathname_pattern(&result)
}

/// Returns true if this reference should be skipped.
pub fn should_skip(pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }
    if pattern.starts_with("http://")
        || pattern.starts_with("https://")
        || pattern.starts_with("//")
    {
        return true;
    }
    if pattern.starts_with('?') || pattern.starts_with('#') {
        return true;
    }
    if pattern.starts_with(":param") {
        return true;
    }
    false
}
