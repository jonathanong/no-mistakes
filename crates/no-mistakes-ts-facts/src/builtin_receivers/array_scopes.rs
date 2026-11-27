#[derive(Default)]
struct ScopeBindings(FxHashSet<String>);
impl<'a> Visit<'a> for ScopeBindings {
    fn visit_binding_identifier(&mut self, id: &BindingIdentifier<'a>) {
        self.0.insert(id.name.to_string());
    }
}
impl ArrayUses {
    fn nested_scope(
        &mut self,
        parameters: &FormalParameters<'_>,
        own_arguments: bool,
        name: Option<&str>,
        scan: impl FnOnce(&mut Self),
    ) {
        let mut bindings = ScopeBindings::default();
        for parameter in &parameters.items {
            bindings.visit_binding_pattern(&parameter.pattern);
        }
        if let Some(rest) = &parameters.rest {
            bindings.visit_binding_pattern(&rest.rest.argument);
        }
        if own_arguments {
            bindings.0.insert("arguments".to_owned());
        }
        if let Some(name) = name {
            bindings.0.insert(name.to_owned());
        }
        let masked = bindings
            .0
            .into_iter()
            .filter(|name| self.candidates.remove(name))
            .collect::<Vec<_>>();
        scan(self);
        self.candidates.extend(masked);
    }
}
