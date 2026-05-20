fn scan_expr(
    expr: &Expression,
    source: &str,
    namespace_imports: &HashMap<String, String>,
    usage: &mut QueueUsage,
) {
    match expr {
        Expression::CallExpression(call) => {
            // Check for <binding>.add(...) or <binding>.addBulk(...)
            if let Some((binding, method)) = extract_member_call(call) {
                if method == "add" {
                    let job = call.arguments.first().and_then(|a| literal_str(a));
                    let line = byte_offset_to_line(source, call.span.start as usize);
                    usage.enqueue_calls.push(EnqueueCall {
                        binding: binding.clone(),
                        job,
                        line,
                    });
                } else if method == "addBulk" {
                    if let Some(Argument::ArrayExpression(arr)) = call.arguments.first() {
                        for el in &arr.elements {
                            if let ArrayExpressionElement::ObjectExpression(obj) = el {
                                for prop in &obj.properties {
                                    if let ObjectPropertyKind::ObjectProperty(p) = prop {
                                        if p.key.static_name().as_deref() == Some("name") {
                                            let job = literal_str_expr(&p.value);
                                            let line = byte_offset_to_line(
                                                source,
                                                call.span.start as usize,
                                            );
                                            usage.enqueue_calls.push(EnqueueCall {
                                                binding: binding.clone(),
                                                job,
                                                line,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check for `new Worker(...)` (handled below in NewExpression).
            // Recurse into call arguments for nested expressions.
            for arg in &call.arguments {
                if let Some(e) = arg.as_expression() {
                    scan_expr(e, source, namespace_imports, usage);
                }
            }
            // Recurse into callee.
            scan_expr(&call.callee, source, namespace_imports, usage);
        }
        Expression::NewExpression(new_expr) => {
            let callee_name = match &new_expr.callee {
                Expression::Identifier(id) => Some(id.name.as_str()),
                _ => None,
            };
            if callee_name == Some("Worker") {
                let queue_name = new_expr.arguments.first().and_then(|a| literal_str(a));

                // Look for `import * as processors from '...'` in namespace imports.
                // Detect it by finding a namespace import whose local name appears in the handler.
                // For simplicity, return any namespace import specifier that could be processors.
                // The handler body is not trivially inspectable here, so we return the first
                // namespace import that looks like processors (ending in /processors.mts or similar).
                let processors_specifier = namespace_imports
                    .values()
                    .find(|s| {
                        let name = s.rsplit('/').next().unwrap_or("");
                        name.starts_with("processors")
                    })
                    .cloned();

                let line = byte_offset_to_line(source, new_expr.span.start as usize);
                usage.worker_declarations.push(WorkerDeclaration {
                    queue_name,
                    processors_specifier,
                    line,
                });
            }
        }
        Expression::ChainExpression(chain) => {
            if let Some(e) = chain.expression.as_member_expression() {
                // Try to scan the object of the member expression
                scan_expr(e.object(), source, namespace_imports, usage);
            }
        }
        Expression::AwaitExpression(a) => {
            scan_expr(&a.argument, source, namespace_imports, usage);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            let oxc::ast::ast::FunctionBody { statements, .. } = arrow.body.as_ref();
            for s in statements {
                scan_stmt(s, source, namespace_imports, usage);
            }
        }
        Expression::TSAsExpression(ts_as) => {
            scan_expr(&ts_as.expression, source, namespace_imports, usage);
        }
        Expression::TSNonNullExpression(ts_nn) => {
            scan_expr(&ts_nn.expression, source, namespace_imports, usage);
        }
        Expression::StaticMemberExpression(member) => {
            scan_expr(&member.object, source, namespace_imports, usage);
        }
        _ => {}
    }
}

/// If `call` is `<identifier>.<method>(...)`, return `(identifier_name, method_name)`.
fn extract_member_call<'a>(call: &'a CallExpression) -> Option<(String, &'a str)> {
    if let Expression::StaticMemberExpression(member) = &call.callee {
        if let Expression::Identifier(obj) = &member.object {
            return Some((obj.name.as_str().to_string(), member.property.name.as_str()));
        }
    }
    None
}

fn literal_str(arg: &Argument) -> Option<String> {
    if let Argument::StringLiteral(s) = arg {
        return Some(s.value.as_str().to_string());
    }
    if let Some(e) = arg.as_expression() {
        return literal_str_expr(e);
    }
    None
}

fn literal_str_expr(expr: &Expression) -> Option<String> {
    if let Expression::StringLiteral(s) = expr {
        return Some(s.value.as_str().to_string());
    }
    None
}

