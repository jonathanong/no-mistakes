fn collect_urls_from_expr(expr: &Expression, urls: &mut Vec<String>) {
    match expr {
        Expression::CallExpression(call) => {
            if let Expression::StaticMemberExpression(member) = &call.callee {
                let method = member.property.name.as_str();
                if method == "goto" {
                    if is_page_receiver(&member.object) {
                        if let Some(url) = route_arg(&call.arguments, 0) {
                            if url.starts_with('/') {
                                urls.push(url);
                            }
                        }
                    }
                } else if method == "click" {
                    if is_page_receiver(&member.object) {
                        if let Some(url) = route_arg(&call.arguments, 0)
                            .as_deref()
                            .and_then(extract_href_from_selector)
                        {
                            urls.push(url);
                        }
                    }
                } else if method == "waitForURL" && is_page_receiver(&member.object) {
                    if let Some(url) =
                        route_arg(&call.arguments, 0).filter(|url| url.starts_with('/'))
                    {
                        urls.push(url);
                    }
                } else if method == "toHaveURL" && is_expect_page_call(&member.object) {
                    if let Some(url) =
                        route_arg(&call.arguments, 0).filter(|url| url.starts_with('/'))
                    {
                        urls.push(url);
                    }
                }
            } else if matches!(&call.callee, Expression::Identifier(callee) if callee.name == "navigateTo")
            {
                for index in [0, 1] {
                    if let Some(url) =
                        route_arg(&call.arguments, index).filter(|url| url.starts_with('/'))
                    {
                        urls.push(url);
                        break;
                    }
                }
            }
            // Recurse into arguments.
            for arg in &call.arguments {
                if let Some(e) = arg.as_expression() {
                    collect_urls_from_expr(e, urls);
                }
            }
        }
        Expression::AwaitExpression(a) => collect_urls_from_expr(&a.argument, urls),
        Expression::ArrowFunctionExpression(arrow) => {
            collect_urls_from_stmts(&arrow.body.statements, urls);
        }
        Expression::ConditionalExpression(c) => {
            collect_urls_from_expr(&c.test, urls);
            collect_urls_from_expr(&c.consequent, urls);
            collect_urls_from_expr(&c.alternate, urls);
        }
        Expression::LogicalExpression(l) => {
            collect_urls_from_expr(&l.left, urls);
            collect_urls_from_expr(&l.right, urls);
        }
        Expression::SequenceExpression(s) => {
            for expr in &s.expressions {
                collect_urls_from_expr(expr, urls);
            }
        }
        _ => {}
    }
}

fn collect_urls_from_for_stmt(f: &ForStatement, urls: &mut Vec<String>) {
    if let Some(init) = &f.init {
        match init {
            ForStatementInit::VariableDeclaration(v) => {
                for decl in &v.declarations {
                    if let Some(init) = &decl.init {
                        collect_urls_from_expr(init, urls);
                    }
                }
            }
            other => {
                if let Some(expr) = other.as_expression() {
                    collect_urls_from_expr(expr, urls);
                }
            }
        }
    }
    if let Some(test) = &f.test {
        collect_urls_from_expr(test, urls);
    }
    if let Some(update) = &f.update {
        collect_urls_from_expr(update, urls);
    }
    collect_urls_from_stmt(&f.body, urls);
}

