use super::escaped_class_lookup::{binding_key, class_of_new, method_callable};
use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_arrow_scope(&mut self) {
        if let Some(id) = self.current_function_id() {
            self.receiver_escapes.arrows.insert(id);
        }
    }

    pub(in super::super::super) fn note_instance_field(
        &mut self,
        class_id: CallableId,
        property: &oxc_ast::ast::PropertyDefinition<'_>,
    ) {
        let Some(name) = crate::codebase::ts_source::static_property_key_name(&property.key) else {
            return;
        };
        let Some(value) = property.value.as_ref() else {
            return;
        };
        let Expression::ArrowFunctionExpression(arrow) =
            crate::codebase::ts_source::unwrap_ts_wrappers(value)
        else {
            return;
        };
        self.receiver_escapes
            .fields
            .insert((class_id, name.to_string()), CallableId(arrow.span.start));
    }

    pub(super) fn note_instance_aliases(&mut self, declarator: &VariableDeclarator<'_>) {
        let Some(init) = &declarator.init else {
            return;
        };
        note_alias(self, declarator, init);
        note_held(self, declarator, init);
        note_method_alias(self, declarator, init);
        note_arguments_alias(self, declarator, init);
    }

    pub(in super::super::super) fn note_value_return(
        &mut self,
        statement: &oxc_ast::ast::ReturnStatement<'_>,
    ) {
        let Some(owner) = self.current_function_id() else {
            return;
        };
        let value = statement
            .argument
            .as_ref()
            .map_or(ValueExpression::Unknown, |argument| {
                if let Some(class_id) = class_of_new(self, argument) {
                    self.receiver_escapes.returns.insert(owner, class_id);
                }
                self.value_expression(argument)
            });
        self.value_flow.returns.push((owner, value));
    }
}

fn note_alias(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'_>,
    init: &Expression<'_>,
) {
    let Expression::Identifier(identifier) = crate::codebase::ts_source::unwrap_ts_wrappers(init)
    else {
        return;
    };
    let Some(class_id) = collector
        .receiver_escapes
        .instances
        .get(&binding_key(collector, identifier.name.as_str()))
        .copied()
    else {
        return;
    };
    for name in binding_names(&declarator.id) {
        collector
            .receiver_escapes
            .instances
            .insert(binding_key(collector, &name), class_id);
    }
}

fn note_held(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'_>,
    init: &Expression<'_>,
) {
    let Expression::ObjectExpression(object) = crate::codebase::ts_source::unwrap_ts_wrappers(init)
    else {
        return;
    };
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(key) = crate::codebase::ts_source::static_property_key_name(&property.key) else {
            continue;
        };
        let Some(class_id) = class_of_new(collector, &property.value)
            .or_else(|| instance_of(collector, &property.value))
        else {
            continue;
        };
        for name in binding_names(&declarator.id) {
            let (scope, holder) = binding_key(collector, &name);
            collector
                .receiver_escapes
                .instances
                .insert((scope, format!("{holder}.{key}")), class_id);
        }
    }
}

fn instance_of(collector: &ImportCollector, expression: &Expression<'_>) -> Option<CallableId> {
    let Expression::Identifier(identifier) =
        crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    else {
        return None;
    };
    collector
        .receiver_escapes
        .instances
        .get(&binding_key(collector, identifier.name.as_str()))
        .copied()
}

fn note_method_alias(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'_>,
    init: &Expression<'_>,
) {
    let Some(method) = method_callable(collector, init) else {
        return;
    };
    for name in binding_names(&declarator.id) {
        collector
            .receiver_escapes
            .method_aliases
            .insert(binding_key(collector, &name), method);
    }
}

fn note_arguments_alias(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'_>,
    init: &Expression<'_>,
) {
    let ValueExpression::Named {
        name, scope: None, ..
    } = collector.value_expression(init)
    else {
        return;
    };
    if name != "arguments" {
        return;
    }
    for binding in binding_names(&declarator.id) {
        collector
            .receiver_escapes
            .argument_aliases
            .insert(binding_key(collector, &binding));
    }
}
