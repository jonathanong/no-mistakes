use super::*;

impl Typed {
    // Indirect/destructured patterns retain possible runtime uses without
    // widening the direct imported-type proof supported by low effort.
    pub(super) fn unknown_pattern(&mut self, pattern: &BindingPattern<'_>, ty: &TSType<'_>) {
        if possible::contains(&self.possible, ty) {
            self.unknown_bindings(pattern);
        }
    }

    fn unknown_bindings(&mut self, pattern: &BindingPattern<'_>) {
        match pattern {
            BindingPattern::BindingIdentifier(binding) => {
                if let Some(id) = self.bindings.get(&span(binding.span)) {
                    self.selected.insert(*id);
                    self.unsupported.insert(*id);
                }
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.unknown_bindings(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.unknown_bindings(&rest.argument);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.unknown_bindings(element);
                }
                if let Some(rest) = &array.rest {
                    self.unknown_bindings(&rest.argument);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.unknown_bindings(&assignment.left);
            }
        }
    }
}
