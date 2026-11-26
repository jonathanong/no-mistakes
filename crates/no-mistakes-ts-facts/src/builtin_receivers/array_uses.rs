#[derive(Default)]
struct ArrayUses {
    candidates: FxHashSet<String>,
    references: FxHashMap<u32, String>,
    safe: FxHashSet<u32>,
    aliases: Vec<(String, String)>,
    global_array: bool,
    forced: FxHashSet<String>,
    local_callback: bool,
    callback_names: FxHashSet<String>,
    write_target: bool,
}

include!("array_scopes.rs");

impl ArrayUses {
    fn instability(self) -> FxHashSet<String> {
        let uses = self;
        let mut unstable = uses
            .references
            .into_iter()
            .filter(|(span, _)| !uses.safe.contains(span))
            .map(|(_, name)| name)
            .collect::<FxHashSet<_>>();
        unstable.extend(uses.forced);
        loop {
            let before = unstable.len();
            for (left, right) in &uses.aliases {
                if unstable.contains(left) || unstable.contains(right) {
                    unstable.insert(left.clone());
                    unstable.insert(right.clone());
                }
            }
            if unstable.len() == before {
                break;
            }
        }
        unstable
    }

    fn record_aliases(&mut self, name: &str, expression: &Expression<'_>) {
        match expression.without_parentheses() {
            Expression::ConditionalExpression(choice) => {
                self.record_aliases(name, &choice.consequent);
                self.record_aliases(name, &choice.alternate);
            }
            Expression::TSAsExpression(cast) => self.record_aliases(name, &cast.expression),
            Expression::TSNonNullExpression(inner) => self.record_aliases(name, &inner.expression),
            expression => {
                if let Some(source) = source_array(expression) {
                    if self.candidates.contains(source) {
                        if matches!(expression, Expression::Identifier(_)) {
                            self.safe_identifier(expression);
                        }
                        self.aliases.push((name.to_owned(), source.to_owned()));
                    }
                }
            }
        }
    }

    fn safe_identifier(&mut self, expression: &Expression<'_>) {
        if let Expression::Identifier(id) = expression.without_parentheses() {
            if self.candidates.contains(id.name.as_str()) {
                self.safe.insert(id.span.start);
            }
        }
    }
}

impl<'a> Visit<'a> for ArrayUses {
    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        let previous = std::mem::replace(&mut self.write_target, true);
        walk::walk_simple_assignment_target(self, target);
        self.write_target = previous;
    }
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if self.local_callback && !self.write_target && member.property.name == "length" {
            self.safe_identifier(&member.object);
        }
        walk::walk_static_member_expression(self, member);
    }
    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if self.local_callback
            && !self.write_target
            && matches!(member.expression.without_parentheses(), Expression::StringLiteral(value) if value.value == "length")
        {
            self.safe_identifier(&member.object);
        }
        walk::walk_computed_member_expression(self, member);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if self.local_callback {
            self.nested_scope(
                &function.params,
                true,
                function.id.as_ref().map(|id| id.name.as_str()),
                |uses| walk::walk_function(uses, function, flags),
            );
        } else {
            walk::walk_function(self, function, flags);
        }
    }
    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.local_callback {
            self.nested_scope(&arrow.params, false, None, |uses| {
                walk::walk_arrow_function_expression(uses, arrow)
            });
        } else {
            walk::walk_arrow_function_expression(self, arrow);
        }
    }

    fn visit_variable_declarator(&mut self, node: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(id), Some(init)) = (&node.id, node.init.as_ref())
        {
            if self.candidates.contains(id.name.as_str())
                || source_array(init).is_some_and(|source| self.candidates.contains(source))
            {
                self.candidates.insert(id.name.to_string());
                self.record_aliases(id.name.as_str(), init);
            }
        }
        walk::walk_variable_declarator(self, node);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() {
            let method = member.property.name.as_str();
            if method == "join" && !self.local_callback {
                self.safe_identifier(&member.object);
            }
            if matches!(
                method,
                "forEach"
                    | "map"
                    | "filter"
                    | "flatMap"
                    | "reduce"
                    | "reduceRight"
                    | "some"
                    | "every"
                    | "find"
                    | "findIndex"
                    | "findLast"
                    | "findLastIndex"
            ) && !self.local_callback
            {
                self.safe_identifier(&member.object);
                if source_array(&member.object)
                    .is_some_and(|source| self.candidates.contains(source))
                {
                    if let Some(Argument::Identifier(id)) = call.arguments.first() {
                        self.callback_names.insert(id.name.to_string());
                    }
                }
            }
            if self.global_array
                && method == "from"
                && matches!(&member.object, Expression::Identifier(id) if id.name == "Array")
            {
                if let Some(iterable) = call.arguments.first().and_then(Argument::as_expression) {
                    self.safe_identifier(iterable);
                }
            }
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_identifier_reference(&mut self, node: &IdentifierReference<'a>) {
        if self.candidates.contains(node.name.as_str()) {
            self.references
                .insert(node.span.start, node.name.to_string());
        }
    }
}
