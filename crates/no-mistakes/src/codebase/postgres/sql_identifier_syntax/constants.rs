use super::*;

impl<'a> Visit<'a> for Constants<'_> {
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declaration in &value.declarations {
                if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                    if !self.wanted.contains(&span(binding.span)) {
                        continue;
                    }
                    let value = declaration.init.as_ref().map_or(Input::Unknown, |value| {
                        input(
                            value,
                            &mut self.syntax.env_references,
                            &self.syntax.template_binds,
                        )
                    });
                    if matches!(value, Input::Sql { .. })
                        && (!matches!(
                            value,
                            Input::Sql {
                                parameterized: Some(_),
                                ..
                            }
                        ) || self.syntax.safe_parameterized.contains(&span(binding.span)))
                    {
                        self.syntax.constants.insert(span(binding.span), value);
                    }
                }
            }
        }
        walk::walk_variable_declaration(self, value);
    }
}
