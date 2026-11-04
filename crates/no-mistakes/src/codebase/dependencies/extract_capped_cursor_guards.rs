#[derive(Default)]
struct CursorGuards {
    bindings: FxHashSet<String>,
    writes: FxHashMap<String, Vec<u32>>,
    opaque: bool,
    escaped: FxHashSet<String>,
}

impl<'a> Visit<'a> for CursorGuards {
    fn visit_binding_identifier(&mut self, identifier: &oxc_ast::ast::BindingIdentifier<'a>) {
        self.bindings.insert(identifier.name.to_string());
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        for (name, _, _) in extract_value_flow::write_targets::targets(&assignment.left) {
            self.writes
                .entry(name)
                .or_default()
                .push(assignment.span.start);
        }
        self.escaped
            .extend(cursor_plain_references(&assignment.right, false));
        walk::walk_assignment_expression(self, assignment);
    }

    fn visit_update_expression(&mut self, update: &oxc_ast::ast::UpdateExpression<'a>) {
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) =
            &update.argument
        {
            self.writes
                .entry(id.name.to_string())
                .or_default()
                .push(update.span.start);
        } else {
            self.opaque = true;
        }
        walk::walk_update_expression(self, update);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.escaped
            .extend(cursor_plain_references(&call.callee, true));
        for argument in &call.arguments {
            if let Some(expression) = argument.as_expression() {
                self.escaped
                    .extend(cursor_plain_references(expression, false));
            } else {
                self.opaque = true;
            }
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_new_expression(&mut self, _: &NewExpression<'a>) {
        self.opaque = true;
    }
    fn visit_unary_expression(&mut self, expression: &oxc_ast::ast::UnaryExpression<'a>) {
        if expression.operator == oxc_ast::ast::UnaryOperator::Delete {
            self.opaque = true;
        }
        walk::walk_unary_expression(self, expression);
    }
    fn visit_tagged_template_expression(&mut self, _: &TaggedTemplateExpression<'a>) {
        self.opaque = true;
    }
    fn visit_for_statement(&mut self, _: &oxc_ast::ast::ForStatement<'a>) {
        self.opaque = true;
    }
    fn visit_for_of_statement(&mut self, _: &oxc_ast::ast::ForOfStatement<'a>) {
        self.opaque = true;
    }
    fn visit_for_in_statement(&mut self, _: &oxc_ast::ast::ForInStatement<'a>) {
        self.opaque = true;
    }
    fn visit_while_statement(&mut self, _: &oxc_ast::ast::WhileStatement<'a>) {
        self.opaque = true;
    }
    fn visit_do_while_statement(&mut self, _: &oxc_ast::ast::DoWhileStatement<'a>) {
        self.opaque = true;
    }
    fn visit_function(&mut self, _: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
        self.opaque = true;
    }
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {
        self.opaque = true;
    }
    fn visit_class(&mut self, _: &Class<'a>) {
        self.opaque = true;
    }
    fn visit_continue_statement(&mut self, _: &oxc_ast::ast::ContinueStatement<'a>) {
        self.opaque = true;
    }
    fn visit_try_statement(&mut self, _: &oxc_ast::ast::TryStatement<'a>) {
        self.opaque = true;
    }
}

fn cursor_result_member(expression: &Expression<'_>, result: &str) -> bool {
    let Expression::StaticMemberExpression(member) = expression else {
        return false;
    };
    matches!(&member.object, Expression::Identifier(id) if id.name == result)
}

fn cursor_argument_reads(expression: &Expression<'_>, cursor: &str) -> bool {
    match expression {
        Expression::Identifier(id) => id.name == cursor,
        Expression::ObjectExpression(object) => {
            cursor_object_keys(expression, &mut FxHashSet::default())
                && object.properties.iter().any(|property| match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        cursor_argument_reads(&property.value, cursor)
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        cursor_argument_reads(&spread.argument, cursor)
                    }
                })
        }
        Expression::ConditionalExpression(conditional) => {
            let consequent = cursor_argument_reads(&conditional.consequent, cursor);
            let alternate = cursor_argument_reads(&conditional.alternate, cursor);
            match literal_truthiness(&conditional.test) {
                Some(true) => consequent,
                Some(false) => alternate,
                None => {
                    consequent
                        && (alternate
                            || matches!(&conditional.test,
                        Expression::Identifier(id) if id.name == cursor))
                }
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            cursor_argument_reads(&parenthesized.expression, cursor)
        }
        _ => false,
    }
}

fn cursor_object_keys(expression: &Expression<'_>, keys: &mut FxHashSet<String>) -> bool {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            cursor_object_keys(&parenthesized.expression, keys)
        }
        Expression::ObjectExpression(object) => {
            object.properties.iter().all(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    !property.computed
                        && property.key.static_name().is_some_and(|key| {
                            key.as_str()
                                .is_some_and(|name| keys.insert(name.to_string()))
                        })
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    cursor_object_keys(&spread.argument, keys)
                }
            })
        }
        Expression::ConditionalExpression(conditional) => {
            cursor_object_keys(&conditional.consequent, keys)
                && cursor_object_keys(&conditional.alternate, keys)
        }
        _ => false,
    }
}
