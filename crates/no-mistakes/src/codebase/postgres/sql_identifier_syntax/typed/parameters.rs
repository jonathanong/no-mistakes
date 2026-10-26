use super::*;
use oxc_ast::ast::{FormalParameters, ObjectPattern, TSSignature};

impl Typed {
    pub(super) fn parameters(&mut self, parameters: &FormalParameters<'_>) {
        for parameter in &parameters.items {
            let ty = parameter
                .type_annotation
                .as_ref()
                .map(|value| &value.type_annotation);
            self.parameter(&parameter.pattern, ty, parameter.initializer.as_deref());
        }
        if let Some(rest) = &parameters.rest {
            if let Some(annotation) = &rest.type_annotation {
                self.unknown_pattern(&rest.rest.argument, &annotation.type_annotation);
            }
        }
    }
    fn parameter(
        &mut self,
        pattern: &BindingPattern<'_>,
        ty: Option<&TSType<'_>>,
        initializer: Option<&Expression<'_>>,
    ) {
        if let Some(initializer) = initializer {
            self.default_parameter(pattern, ty, initializer);
            return;
        }
        match (pattern, ty) {
            (BindingPattern::AssignmentPattern(default), ty) => {
                self.default_parameter(&default.left, ty, &default.right);
            }
            (BindingPattern::BindingIdentifier(binding), Some(ty)) => self.record(binding, ty),
            (BindingPattern::ObjectPattern(object), ty) => self.object_parameter(object, ty),
            (pattern, Some(ty)) => self.unknown_pattern(pattern, ty),
            _ => {}
        }
    }
    fn default_parameter(
        &mut self,
        pattern: &BindingPattern<'_>,
        ty: Option<&TSType<'_>>,
        initializer: &Expression<'_>,
    ) {
        let known = self.default_source(initializer);
        if let Some(at) = known {
            self.transports.insert(at);
        }
        if let (BindingPattern::BindingIdentifier(binding), Some(ty)) = (pattern, ty) {
            if known.is_some() {
                self.record(binding, ty);
                if !self
                    .bindings
                    .get(&span(binding.span))
                    .is_some_and(|id| self.selected.contains(id))
                {
                    self.unknown_bindings(pattern);
                }
            } else {
                self.unknown_pattern(pattern, ty);
            }
        } else if known.is_some() {
            self.unknown_bindings(pattern);
        } else if let Some(ty) = ty {
            self.unknown_pattern(pattern, ty);
        }
    }
    fn object_parameter(&mut self, object: &ObjectPattern<'_>, ty: Option<&TSType<'_>>) {
        for property in &object.properties {
            let Some(key) = property.key.static_name() else {
                // An unresolved key retains possible uses without proving an executor.
                if let Some(ty) = ty {
                    self.unknown_pattern(&property.value, ty);
                }
                continue;
            };
            let member = match ty {
                Some(TSType::TSTypeLiteral(literal)) => literal.members.iter().find_map(|member| {
                    let TSSignature::TSPropertySignature(signature) = member else {
                        return None;
                    };
                    (signature.key.static_name().as_deref() == Some(key.as_ref()))
                        .then(|| {
                            signature
                                .type_annotation
                                .as_ref()
                                .map(|value| &value.type_annotation)
                        })
                        .flatten()
                }),
                // Only the existing query member of a configured receiver is an executor.
                Some(ty) if key == "query" && self.direct_type(ty) => Some(ty),
                _ => None,
            };
            if let Some(member) = member {
                if matches!(
                    property.value,
                    BindingPattern::BindingIdentifier(_) | BindingPattern::AssignmentPattern(_)
                ) {
                    self.parameter(&property.value, Some(member), None);
                } else {
                    self.unknown_pattern(&property.value, member);
                }
            } else {
                // Known defaults still retain untyped/unconfigured actual call sites.
                self.parameter(&property.value, None, None);
                if let Some(ty) = ty {
                    self.unknown_pattern(&property.value, ty);
                }
            }
        }
        if let (Some(rest), Some(ty)) = (&object.rest, ty) {
            self.unknown_pattern(&rest.argument, ty);
        }
    }
}
