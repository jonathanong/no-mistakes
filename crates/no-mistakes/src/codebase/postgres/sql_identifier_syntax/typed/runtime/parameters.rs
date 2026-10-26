use super::*;
use oxc_ast::ast::FormalParameters;

impl Runtime<'_> {
    pub(super) fn parameter_properties(&mut self, parameters: &FormalParameters<'_>) {
        for parameter in &parameters.items {
            if parameter.accessibility.is_none() && !parameter.readonly && !parameter.r#override {
                continue;
            }
            if let (BindingPattern::BindingIdentifier(binding), Some(annotation)) =
                (&parameter.pattern, &parameter.type_annotation)
            {
                if possible::contains(self.possible, &annotation.type_annotation) {
                    self.fields
                        .entry((false, false))
                        .or_default()
                        .insert(binding.name.to_string());
                }
            }
        }
    }
}
