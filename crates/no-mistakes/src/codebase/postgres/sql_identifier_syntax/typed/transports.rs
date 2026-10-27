use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use oxc_ast::ast::{FormalParameters, VariableDeclarator};

// False = configured direct executor; true = configured executor factory.
pub(super) fn sources(
    module: &TypeScriptModuleFacts,
    selection: &Selection,
) -> FxHashMap<Span, bool> {
    module
        .bindings
        .iter()
        .filter_map(|binding| {
            if selection.factories.contains(&binding.id) {
                Some((binding, true))
            } else {
                selection
                    .selected
                    .contains(&binding.id)
                    .then_some((binding, false))
            }
        })
        .flat_map(|(binding, factory)| {
            binding
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .map(move |usage| ((usage.span.start, usage.span.end), factory))
        })
        .collect()
}

impl Typed {
    pub(super) fn default_source(&self, expression: &Expression<'_>) -> Option<Span> {
        if let Expression::Identifier(id) = unwrap_ts_wrappers(expression) {
            let at = span(id.span);
            if self.sources.get(&at) == Some(&false) {
                return Some(at);
            }
        }
        self.factory_source(expression)
    }
    fn factory_source(&self, expression: &Expression<'_>) -> Option<Span> {
        let expression = unwrap_ts_wrappers(expression);
        let expression = if let Expression::AwaitExpression(value) = expression {
            unwrap_ts_wrappers(&value.argument)
        } else {
            expression
        };
        let Expression::CallExpression(call) = expression else {
            return None;
        };
        let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        let at = span(id.span);
        (self.sources.get(&at) == Some(&true)).then_some(at)
    }
    pub(super) fn factory_binding(&mut self, variable: &VariableDeclarator<'_>) {
        let Some(at) = variable
            .init
            .as_ref()
            .and_then(|init| self.factory_source(init))
        else {
            return;
        };
        if let BindingPattern::BindingIdentifier(binding) = &variable.id {
            if let Some(id) = self.bindings.get(&span(binding.span)) {
                self.transports.insert(at);
                self.selected.insert(*id);
                self.factory_bindings.insert(*id, self.immutable);
            }
        } else if let BindingPattern::ObjectPattern(object) = &variable.id {
            if object.properties.len() == 1
                && object.rest.is_none()
                && crate::ast::property_key_name(&object.properties[0].key).as_deref()
                    == Some("query")
            {
                self.unknown_bindings(&object.properties[0].value);
                self.transports.insert(at);
            }
        }
    }
    pub(super) fn write(&mut self, at: Span) {
        if let Some(id) = self.bindings.get(&at) {
            self.writes.insert(*id);
            self.transports.insert(at);
        }
    }
    pub(super) fn finish_transports(&mut self) {
        for (id, immutable) in &self.factory_bindings {
            if *immutable {
                self.unsupported.remove(id);
            } else {
                self.unsupported.insert(*id);
            }
        }
        self.unsupported
            .extend(self.writes.intersection(&self.selected).copied());
    }
    pub(super) fn finish_factory_arguments(&mut self) {
        for at in &self.factory_arguments {
            let known = self.sources.contains_key(at)
                || self
                    .bindings
                    .get(at)
                    .is_some_and(|id| self.selected.contains(id));
            if known {
                self.transports.insert(*at);
            }
        }
    }
    pub(super) fn wrapper(&mut self, call: &CallExpression<'_>) {
        let factory = match unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(callee) => {
                let at = span(callee.span);
                (self.sources.get(&at) == Some(&true)).then_some(at)
            }
            _ => None,
        };
        let mut typed_callback = false;
        for argument in &call.arguments {
            let parameters = match argument.as_expression().map(unwrap_ts_wrappers) {
                Some(Expression::ArrowFunctionExpression(value)) => Some(&value.params),
                Some(Expression::FunctionExpression(value)) => Some(&value.params),
                _ => None,
            };
            if let Some(parameters) = parameters {
                typed_callback |= parameters.items.iter().any(|parameter| {
                    parameter
                        .type_annotation
                        .as_ref()
                        .is_some_and(|annotation| self.direct_type(&annotation.type_annotation))
                });
                if let Some(at) = factory {
                    self.transports.insert(at);
                    self.callback(parameters);
                }
            }
        }
        // A typed callback proves its own receiver, not an opaque helper's uses.
        // Only an explicitly configured factory can own companion transport.
        if typed_callback && factory.is_some() {
            if let Some((root, _)) = chain(&call.callee) {
                self.transports.insert(root);
            }
        }
        if factory.is_some() {
            self.factory_arguments
                .extend(call.arguments.iter().filter_map(|argument| {
                    argument
                        .as_expression()
                        .and_then(chain)
                        .map(|(root, _)| root)
                }));
        }
    }
    fn callback(&mut self, parameters: &FormalParameters<'_>) {
        for parameter in &parameters.items {
            if parameter.type_annotation.is_none() {
                self.unknown_bindings(&parameter.pattern);
            }
        }
    }
}
