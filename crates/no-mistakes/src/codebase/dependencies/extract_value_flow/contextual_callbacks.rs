use super::*;
use oxc_ast::ast::{ObjectPropertyKind, TSType, TSTypeOperatorOperator};

fn element_type<'a, 'b>(annotation: &'b TSType<'a>) -> &'b TSType<'a> {
    match annotation {
        TSType::TSTypeOperatorType(operator)
            if operator.operator == TSTypeOperatorOperator::Readonly =>
        {
            element_type(&operator.type_annotation)
        }
        TSType::TSArrayType(array) => &array.element_type,
        TSType::TSTypeReference(reference)
            if matches!(
                type_reference_name(reference).as_deref(),
                Some("Array" | "ReadonlyArray")
            ) =>
        {
            reference
                .type_arguments
                .as_ref()
                .filter(|arguments| arguments.params.len() == 1)
                .map_or(annotation, |arguments| &arguments.params[0])
        }
        _ => annotation,
    }
}

impl ImportCollector {
    /// Records the declared callback type of each direct function property in an
    /// annotated array or object literal so unannotated parameters can inherit
    /// executor types positionally (docs/rules/query-reached-per-item.md).
    pub(super) fn note_contextual_callbacks(&mut self, declarator: &VariableDeclarator<'_>) {
        let (Some(annotation), Some(init)) = (&declarator.type_annotation, &declarator.init) else {
            return;
        };
        let shape = self.annotation_shape(element_type(&annotation.type_annotation));
        if matches!(shape, ValueType::Unknown | ValueType::Number) {
            return;
        }
        let objects: Vec<&ObjectExpression<'_>> = match init.get_inner_expression() {
            Expression::ArrayExpression(array) => array
                .elements
                .iter()
                .filter_map(
                    |element| match element.as_expression()?.get_inner_expression() {
                        Expression::ObjectExpression(object) => Some(&**object),
                        _ => None,
                    },
                )
                .collect(),
            Expression::ObjectExpression(object) => vec![&**object],
            _ => return,
        };
        for object in objects {
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                let Some(name) =
                    crate::codebase::ts_source::static_property_key_name(&property.key)
                else {
                    continue;
                };
                if property.computed {
                    continue;
                }
                let start = match property.value.get_inner_expression() {
                    Expression::ArrowFunctionExpression(function) => function.params.span.start,
                    Expression::FunctionExpression(function) => function.params.span.start,
                    _ => continue,
                };
                self.value_flow.contextual_callbacks.insert(
                    start,
                    ValueType::Member(Box::new(shape.clone()), name.to_string()),
                );
            }
        }
    }

    pub(super) fn contextual_parameter_type(
        &mut self,
        params: &FormalParameters<'_>,
        index: usize,
    ) -> Option<ValueType> {
        let callback = self
            .value_flow
            .contextual_callbacks
            .get(&params.span.start)?;
        Some(ValueType::Param(Box::new(callback.clone()), index))
    }
}
