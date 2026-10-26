use super::*;
use oxc_ast::ast::SimpleAssignmentTarget;
mod assignments;
mod calls;
mod containers;
mod patterns;
use patterns::lower_pattern;
impl Tracker {
    pub fn expression(&mut self, expression: &Expression<'_>, ids: &FxHashMap<Span, usize>) {
        if !self.enabled {
            return;
        }
        let at = span(expression.span());
        let node = match expression {
            Expression::ThisExpression(_) => Node::This(self.frame.this_scope),
            Expression::Identifier(id) => self.record_reference(id, ids),
            Expression::StaticMemberExpression(value) => Node::Read(
                span(value.object.span()),
                Some(value.property.name.to_string()),
            ),
            Expression::ComputedMemberExpression(value) => {
                Node::Read(span(value.object.span()), static_key(&value.expression))
            }
            Expression::UnaryExpression(value) => Node::Data(vec![span(value.argument.span())]),
            Expression::BinaryExpression(value) => {
                Node::Data(vec![span(value.left.span()), span(value.right.span())])
            }
            Expression::AwaitExpression(value) => Node::Copy(span(value.argument.span())),
            Expression::TSNonNullExpression(value) => Node::Copy(span(value.expression.span())),
            Expression::LogicalExpression(value) => {
                Node::Union(vec![span(value.left.span()), span(value.right.span())])
            }
            Expression::ConditionalExpression(value) => {
                self.tests.push(span(value.test.span()));
                Node::Union(vec![
                    span(value.consequent.span()),
                    span(value.alternate.span()),
                ])
            }
            Expression::AssignmentExpression(value) => assignments::record(self, value, ids),
            Expression::ParenthesizedExpression(value) => Node::Copy(span(value.expression.span())),
            Expression::TSAsExpression(value) => Node::Copy(span(value.expression.span())),
            Expression::TSSatisfiesExpression(value) => Node::Copy(span(value.expression.span())),
            Expression::TSTypeAssertion(value) => Node::Copy(span(value.expression.span())),
            Expression::ObjectExpression(object) => {
                containers::methods(self, object);
                containers::object(object)
            }
            Expression::ArrayExpression(array) => containers::array(array),
            Expression::ChainExpression(value) => self.record_chain(&value.expression, ids),
            Expression::CallExpression(call) => self.record_call(call, ids),
            _ => Node::Scalar,
        };
        match &node {
            Node::This(Some(scope)) => self.this_reads.entry(*scope).or_default().push(at),
            Node::Read(object, _) => self.member_reads.entry(*object).or_default().push(at),
            _ => {}
        }
        self.nodes.insert(at, node);
        self.order.push(at);
    }
    pub fn variable(
        &mut self,
        variable: &oxc_ast::ast::VariableDeclarator<'_>,
        ids: &FxHashMap<Span, usize>,
    ) {
        if !self.enabled {
            return;
        }
        if let Some(init) = &variable.init {
            lower_pattern(self, &variable.id, span(init.span()), Vec::new(), ids);
            if let BindingPattern::BindingIdentifier(binding) = &variable.id {
                if let Some(id) = ids.get(&span(binding.span)) {
                    match unwrap_ts_wrappers(init) {
                        Expression::ArrowFunctionExpression(value) => {
                            self.owners.insert(span(value.span), *id);
                        }
                        Expression::FunctionExpression(value) => {
                            self.owners.insert(span(value.span), *id);
                            if let Some(name) = &value.id {
                                if let Some(inner) = ids.get(&span(name.span)) {
                                    self.aliases.push((*id, *inner));
                                }
                            }
                        }
                        Expression::Identifier(value) => {
                            if let Some(source) = ids.get(&span(value.span)) {
                                self.aliases.push((*source, *id));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    pub fn global_write(&mut self, name: &str, at: Span, ids: &FxHashMap<Span, usize>) {
        if !self.enabled {
            return;
        }
        if matches!(name, "Object" | "globalThis") && !ids.contains_key(&at) {
            self.global_escape = true;
        }
    }
    pub fn write(&mut self, target: &SimpleAssignmentTarget<'_>, ids: &FxHashMap<Span, usize>) {
        if !self.enabled {
            return;
        }
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = target {
            self.global_write(id.name.as_str(), span(id.span), ids);
        }
        if let Some(root) = super::super::template_binds::assignment_root(target) {
            self.write_roots.insert(root);
        }
    }
}
fn freeze_object(call: &CallExpression<'_>, ids: &FxHashMap<Span, usize>) -> Option<Span> {
    if call.arguments.len() != 1 || call.arguments[0].as_expression().is_none() {
        return None;
    }
    let (root, parts) = chain(&call.callee)?;
    (parts == ["Object", "freeze"] && !ids.contains_key(&root)).then_some(root)
}
fn static_key(expression: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(key) => Some(crate::ast::js_literal(&key).into_owned()),
        Expression::NumericLiteral(key) => numeric_key(key.value),
        _ => None,
    }
}

fn numeric_key(value: f64) -> Option<String> {
    (value.is_finite() && (0.0..=9_007_199_254_740_991.0).contains(&value) && value.fract() == 0.0)
        .then(|| format!("{value:.0}"))
}
fn property_key(key: &oxc_ast::ast::PropertyKey<'_>) -> Option<String> {
    use oxc_ast::ast::PropertyKey;
    match key {
        PropertyKey::StaticIdentifier(value) => Some(value.name.to_string()),
        PropertyKey::StringLiteral(value) => Some(crate::ast::js_literal(&value).into_owned()),
        PropertyKey::NumericLiteral(value) => numeric_key(value.value),
        PropertyKey::NullLiteral(_) => Some("null".into()),
        _ => None,
    }
}
