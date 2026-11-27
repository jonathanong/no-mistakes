// Decorator expressions and computed keys precede member application. Defer
// only eager static children; each AST child is still visited exactly once.
enum DeferredDecoratedStatic<'ast, 'node> {
    Value(&'node Expression<'ast>),
    Block(&'node StaticBlock<'ast>),
}

fn defer_decorated_static<'ast, 'node>(
    collector: &mut ImportCollector,
    class: Option<(&str, CallableId)>,
    element: &'node ClassElement<'ast>,
) -> Option<DeferredDecoratedStatic<'ast, 'node>> {
    match element {
        ClassElement::StaticBlock(block) => Some(DeferredDecoratedStatic::Block(block)),
        ClassElement::PropertyDefinition(property)
            if property.r#static && static_callable_field(property).is_none() =>
        {
            let value = property.value.as_ref()?;
            if let Some((name, id)) = class {
                record_class_computed_key_symbol_reference(collector, name, id, &property.key);
            }
            walk_decorators_as_invocations(collector, &property.decorators);
            collector.visit_property_key(&property.key);
            if let Some(annotation) = &property.type_annotation {
                collector.visit_ts_type_annotation(annotation);
            }
            Some(DeferredDecoratedStatic::Value(value))
        }
        ClassElement::AccessorProperty(property) if property.r#static => {
            let value = property.value.as_ref()?;
            walk_decorators_as_invocations(collector, &property.decorators);
            collector.visit_property_key(&property.key);
            if let Some(annotation) = &property.type_annotation {
                collector.visit_ts_type_annotation(annotation);
            }
            Some(DeferredDecoratedStatic::Value(value))
        }
        _ => None,
    }
}

fn visit_deferred_decorated_static<'a>(
    collector: &mut ImportCollector,
    pending: Vec<DeferredDecoratedStatic<'a, '_>>,
) {
    for initializer in pending {
        match initializer {
            DeferredDecoratedStatic::Value(value) => {
                visit_receiver_field_initializer(collector, true, value);
            }
            DeferredDecoratedStatic::Block(block) => collector.visit_static_block(block),
        }
    }
}

fn walk_anonymous_member_decorated_class<'a>(collector: &mut ImportCollector, class: &Class<'a>) {
    walk::walk_decorators(collector, &class.decorators);
    if let Some(parameters) = &class.type_parameters {
        collector.visit_ts_type_parameter_declaration(parameters);
    }
    if let Some(heritage) = &class.heritage {
        collector.visit_class_heritage(heritage);
    }
    collector.visit_ts_class_implements_list(&class.implements);
    let mut pending = Vec::new();
    for element in &class.body.body {
        if let Some(initializer) = defer_decorated_static(collector, None, element) {
            pending.push(initializer);
        } else {
            collector.visit_class_element(element);
        }
    }
    visit_deferred_decorated_static(collector, pending);
}
