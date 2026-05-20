fn collect_urls_from_body(body: Option<&FunctionBody>, urls: &mut Vec<String>) {
    if let Some(body) = body {
        collect_urls_from_stmts(&body.statements, urls);
    }
}

fn collect_urls_from_stmts(statements: &[Statement], urls: &mut Vec<String>) {
    for stmt in statements {
        collect_urls_from_stmt(stmt, urls);
    }
}

fn is_page_receiver(expr: &Expression) -> bool {
    matches!(expr, Expression::Identifier(id) if id.name.as_str() == "page")
}

fn is_expect_page_call(expr: &Expression) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return false;
    };
    if callee.name.as_str() != "expect" {
        return false;
    }
    matches!(
        call.arguments.first().and_then(|arg| arg.as_expression()),
        Some(Expression::Identifier(id)) if id.name.as_str() == "page"
    )
}

fn route_arg(args: &[Argument], index: usize) -> Option<String> {
    let arg = args.get(index)?;
    match arg {
        Argument::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Argument::TemplateLiteral(tpl) => Some(normalize_template(tpl)),
        _ => arg.as_expression().and_then(route_expr),
    }
}

fn route_expr(expr: &Expression) -> Option<String> {
    match expr {
        Expression::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Expression::TemplateLiteral(tpl) => Some(normalize_template(tpl)),
        Expression::ParenthesizedExpression(paren) => route_expr(&paren.expression),
        Expression::NewExpression(new_expr) => {
            let Expression::Identifier(callee) = &new_expr.callee else {
                return None;
            };
            if callee.name.as_str() != "RegExp" {
                return None;
            }
            route_arg(&new_expr.arguments, 0)
        }
        _ => None,
    }
}

/// Parse `a[href="/users/42"]` or `a[href='/users/42']` → `/users/42`.
fn extract_href_from_selector(selector: &str) -> Option<String> {
    let start = selector.find("[href=")?;
    let rest = &selector[start + 6..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &rest[quote.len_utf8()..];
    let end = rest.find(quote)?;
    let url = &rest[..end];
    if url.starts_with('/') {
        Some(url.to_string())
    } else {
        None
    }
}
