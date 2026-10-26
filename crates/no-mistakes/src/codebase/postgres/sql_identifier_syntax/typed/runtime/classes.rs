use super::*;
use oxc_ast::ast::{Class, ClassElement, MethodDefinitionKind};

pub(super) fn visit(runtime: &mut Runtime<'_>, class: &Class<'_>) {
    // Heritage, decorators, and computed keys evaluate with enclosing this.
    runtime.visit_decorators(&class.decorators);
    if let Some(heritage) = &class.heritage {
        runtime.visit_expression(&heritage.expression);
    }
    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method) => {
                runtime.visit_decorators(&method.decorators);
                runtime.visit_property_key(&method.key);
            }
            ClassElement::PropertyDefinition(property) => {
                runtime.visit_decorators(&property.decorators);
                runtime.visit_property_key(&property.key);
            }
            ClassElement::AccessorProperty(property) => {
                runtime.visit_decorators(&property.decorators);
                runtime.visit_property_key(&property.key);
            }
            _ => {}
        }
    }
    let fields = std::mem::take(&mut runtime.fields);
    let methods = std::mem::take(&mut runtime.methods);
    let enabled = runtime.enabled;
    let object_fields = runtime.this_fields.take();
    for element in &class.body.body {
        match element {
            ClassElement::PropertyDefinition(property) => {
                if let (Some(name), Some(annotation)) =
                    (field_name(&property.key), &property.type_annotation)
                {
                    if possible::contains(runtime.possible, &annotation.type_annotation) {
                        runtime
                            .fields
                            .entry((property.r#static, name.0))
                            .or_default()
                            .insert(name.1);
                    }
                }
            }
            ClassElement::AccessorProperty(property) => {
                if let (Some(name), Some(annotation)) =
                    (field_name(&property.key), &property.type_annotation)
                {
                    if possible::contains(runtime.possible, &annotation.type_annotation) {
                        runtime
                            .fields
                            .entry((property.r#static, name.0))
                            .or_default()
                            .insert(name.1);
                    }
                }
            }
            ClassElement::MethodDefinition(method) => {
                runtime
                    .methods
                    .insert(method.value.span.start, method.r#static);
                if method.kind == MethodDefinitionKind::Constructor {
                    runtime.parameter_properties(&method.value.params);
                }
            }
            _ => {}
        }
    }
    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method) => {
                runtime.visit_function(&method.value, oxc_syntax::scope::ScopeFlags::Function);
            }
            ClassElement::PropertyDefinition(property) => {
                runtime.enabled = Some(property.r#static);
                if let Some(value) = &property.value {
                    runtime.visit_expression(value);
                }
            }
            ClassElement::AccessorProperty(property) => {
                runtime.enabled = Some(property.r#static);
                if let Some(value) = &property.value {
                    runtime.visit_expression(value);
                }
            }
            ClassElement::StaticBlock(block) => runtime.visit_static_block(block),
            _ => {}
        }
    }
    runtime.fields = fields;
    runtime.methods = methods;
    runtime.enabled = enabled;
    runtime.this_fields = object_fields;
}
