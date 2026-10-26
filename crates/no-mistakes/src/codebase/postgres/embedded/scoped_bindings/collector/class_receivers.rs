use super::ScopeCollector;
use oxc_ast::ast::{
    BindingPattern, Class, ClassElement, FormalParameter, Function, PropertyDefinition,
};

impl ScopeCollector {
    pub(super) fn prepare_class_receivers(&mut self, class: &Class<'_>) {
        self.found
            .undeclared_this
            .push((class.span, Default::default()));
        for element in &class.body.body {
            if let ClassElement::MethodDefinition(method) = element {
                self.class_methods.insert(method.value.span.start);
                if method.r#static {
                    self.found
                        .undeclared_this_exclusions
                        .push(method.value.span);
                }
            }
        }
    }

    pub(super) fn exclude_dynamic_this(&mut self, function: &Function<'_>) {
        if !self.classes.is_empty() && !self.class_methods.contains(&function.span.start) {
            self.found.undeclared_this_exclusions.push(function.span);
        }
    }

    pub(super) fn record_class_property(&mut self, property: &PropertyDefinition<'_>) {
        if property.r#static {
            self.found.undeclared_this_exclusions.push(property.span);
            return;
        }
        if let (Some(scope), Some(name), Some(annotation)) = (
            self.classes.last().copied(),
            property.key.static_name(),
            &property.type_annotation,
        ) {
            self.record_path_annotation(
                "this",
                scope.start,
                &annotation.type_annotation,
                &format!(".{name}"),
                Some(scope),
            );
        }
    }

    pub(super) fn record_parameter_property(&mut self, parameter: &FormalParameter<'_>) {
        if parameter.accessibility.is_none() && !parameter.readonly && !parameter.r#override {
            return;
        }
        if let (Some(scope), BindingPattern::BindingIdentifier(name), Some(annotation)) = (
            self.classes.last().copied(),
            &parameter.pattern,
            &parameter.type_annotation,
        ) {
            self.record_path_annotation(
                "this",
                scope.start,
                &annotation.type_annotation,
                &format!(".{}", name.name),
                Some(scope),
            );
        }
    }
}
