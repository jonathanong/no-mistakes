use super::*;

pub(super) fn direct_arrow(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => {
            direct_arrow(&parenthesized.expression)
        }
        _ => false,
    }
}

impl ImportCollector {
    pub(super) fn parameter_type_reference(
        &self,
        param: &oxc_ast::ast::FormalParameter<'_>,
    ) -> Option<ValueType> {
        Some(self.annotation_shape(&param.type_annotation.as_ref()?.type_annotation))
    }

    pub(in super::super) fn annotation_shape(
        &self,
        annotation: &oxc_ast::ast::TSType<'_>,
    ) -> ValueType {
        use oxc_ast::ast::TSType;
        match annotation {
            TSType::TSTypeReference(reference) => {
                let Some(name) = type_reference_name(reference) else {
                    return ValueType::Unknown;
                };
                let root = name.split('.').next().unwrap_or(&name);
                if self.type_parameter_shadows(root)
                    || self
                        .type_local_stack
                        .iter()
                        .skip(1)
                        .any(|scope| scope.contains(root))
                {
                    return ValueType::Unknown;
                }
                if name == "NonNullable"
                    && !self.hoisted_non_nullable_type_import
                    && !self.type_binding_shadows(root)
                    && !self.imported_bindings.contains(root)
                    && !self.predeclared_imported_bindings.contains(root)
                {
                    return reference
                        .type_arguments
                        .as_ref()
                        .filter(|arguments| arguments.params.len() == 1)
                        .map_or(ValueType::Unknown, |arguments| {
                            self.annotation_shape(&arguments.params[0])
                        });
                }
                ValueType::Reference(name)
            }
            TSType::TSIndexedAccessType(indexed) => {
                if let TSType::TSLiteralType(literal) = &indexed.index_type {
                    if let oxc_ast::ast::TSLiteral::StringLiteral(key) = &literal.literal {
                        return ValueType::Member(
                            Box::new(self.annotation_shape(&indexed.object_type)),
                            crate::ast::js_literal(key).into_owned(),
                        );
                    }
                }
                ValueType::Unknown
            }
            TSType::TSTypeLiteral(literal) => {
                let mut properties = fx_map();
                for member in &literal.members {
                    if let oxc_ast::ast::TSSignature::TSPropertySignature(property) = member {
                        if !property.computed {
                            if let (Some(key), Some(annotation)) = (
                                crate::codebase::ts_source::static_property_key_name(&property.key),
                                &property.type_annotation,
                            ) {
                                properties.insert(
                                    key.to_string(),
                                    self.annotation_shape(&annotation.type_annotation),
                                );
                            }
                        }
                    }
                }
                ValueType::Object(properties)
            }
            _ => ValueType::Unknown,
        }
    }
}
