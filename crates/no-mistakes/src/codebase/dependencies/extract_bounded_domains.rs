type BoundedGuard = Vec<(usize, String)>;

#[derive(Clone, Copy)]
enum BoundedValue {
    Unknown,
    Number(f64),
    Tuple,
    Sequence,
    Keys,
    Enum,
}

#[derive(Default)]
struct BoundedDomains {
    values: FxHashMap<usize, FxHashMap<String, BoundedValue>>,
    types: FxHashMap<usize, FxHashSet<String>>,
    imported_record: bool,
    imported_object: bool,
}

impl ImportCollector {
    fn bounded_value(&self, name: &str) -> Option<BoundedValue> {
        self.bounded_value_binding(name).map(|(_, value)| value)
    }

    fn bounded_value_binding(&self, name: &str) -> Option<(usize, BoundedValue)> {
        for (depth, scope) in self.lexical_scope_ids.iter().enumerate().rev() {
            if let Some(value) = self
                .bounded_domains
                .values
                .get(scope)
                .and_then(|map| map.get(name))
            {
                return Some((*scope, *value));
            }
            if self.local_stack[depth].contains(name) {
                return None;
            }
        }
        None
    }

    fn bounded_number(&self, expression: &Expression<'_>) -> Option<f64> {
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::NumericLiteral(number)
                if number.value.is_finite()
                    && number.value.fract() == 0.0
                    && number.value.abs() <= 9_007_199_254_740_991.0 =>
            {
                Some(number.value)
            }
            Expression::UnaryExpression(unary)
                if unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation =>
            {
                self.bounded_number(&unary.argument).map(|value| -value)
            }
            Expression::Identifier(id) => match self.bounded_value(id.name.as_str())? {
                BoundedValue::Number(value) => Some(value),
                _ => None,
            },
            _ => None,
        }
    }

    fn record_bounded_declaration(&mut self, declaration: &VariableDeclaration<'_>) {
        if !self.collect_per_item_calls || declaration.kind != VariableDeclarationKind::Const {
            return;
        }
        for declarator in &declaration.declarations {
            let Some(name) = binding_identifier_name(&declarator.id) else {
                continue;
            };
            let value = declarator
                .init
                .as_ref()
                .and_then(|init| {
                    self.bounded_number(init)
                        .map(BoundedValue::Number)
                        .or_else(|| {
                            if literal_const_tuple(init) {
                                Some(BoundedValue::Tuple)
                            } else if literal_fixed_tuple(init) {
                                Some(BoundedValue::Sequence)
                            } else {
                                None
                            }
                        })
                })
                .or_else(|| {
                    declarator
                        .type_annotation
                        .as_ref()
                        .filter(|annotation| self.closed_record(&annotation.type_annotation))
                        .map(|_| BoundedValue::Keys)
                });
            if let Some(value) = value {
                self.bounded_domains
                    .values
                    .entry(self.current_lexical_scope_id())
                    .or_default()
                    .entry(name.to_string())
                    .or_insert(value);
            }
        }
    }

    fn bounded_binding_guard(&self, name: &str, keys: bool) -> Option<BoundedGuard> {
        let (scope, value) = self.bounded_value_binding(name)?;
        let closed = match value {
            BoundedValue::Keys | BoundedValue::Enum => keys,
            BoundedValue::Tuple | BoundedValue::Sequence => !keys,
            _ => false,
        };
        closed.then(|| vec![(scope, name.to_string())])
    }

    fn closed_iteration(&self, expression: &Expression<'_>) -> Option<BoundedGuard> {
        if literal_fixed_tuple(expression) {
            return Some(Vec::new());
        }
        match expression {
            Expression::Identifier(id) => self.bounded_binding_guard(id.name.as_str(), false),
            Expression::CallExpression(call) if self.builtin_bounded_object() => {
                let callee = simple_callee_name(&call.callee)?;
                if !matches!(
                    callee.as_str(),
                    "Object.entries" | "Object.keys" | "Object.values"
                ) || call.arguments.len() != 1
                {
                    return None;
                }
                let Some(Expression::Identifier(id)) = call.arguments[0].as_expression() else {
                    return None;
                };
                self.bounded_binding_guard(id.name.as_str(), true)
            }
            _ => None,
        }
    }
}

fn literal_const_tuple(expression: &Expression<'_>) -> bool {
    let Expression::TSAsExpression(assertion) = expression else {
        return false;
    };
    let oxc_ast::ast::TSType::TSTypeReference(reference) = &assertion.type_annotation else {
        return false;
    };
    if type_reference_name(reference).as_deref() != Some("const") {
        return false;
    }
    let Expression::ArrayExpression(array) = &assertion.expression else {
        return false;
    };
    array.elements.iter().all(|element| {
        element.as_expression().is_some_and(|item| {
            matches!(
                item,
                Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BooleanLiteral(_)
            )
        })
    })
}

fn literal_fixed_tuple(expression: &Expression<'_>) -> bool {
    let Expression::TSAsExpression(assertion) = expression else {
        return false;
    };
    let oxc_ast::ast::TSType::TSTypeReference(reference) = &assertion.type_annotation else {
        return false;
    };
    if type_reference_name(reference).as_deref() != Some("const") {
        return false;
    }
    let Expression::ArrayExpression(array) = &assertion.expression else {
        return false;
    };
    array
        .elements
        .iter()
        .all(|element| element.as_expression().is_some())
}
