use super::*;

#[derive(Clone)]
pub(super) struct Parameters {
    positions: FxHashSet<usize>,
    rest: bool,
    arguments: bool,
}
#[derive(Default)]
struct Bindings(FxHashSet<String>);
impl<'a> Visit<'a> for Bindings {
    fn visit_binding_identifier(&mut self, id: &BindingIdentifier<'a>) {
        self.0.insert(id.name.to_string());
    }
}
impl Parameters {
    pub(super) fn function(function: &Function<'_>) -> Self {
        Self::collect(&function.params, true, |uses| {
            if let Some(body) = &function.body {
                uses.visit_function_body(body);
            }
        })
    }
    pub(super) fn arrow(arrow: &ArrowFunctionExpression<'_>) -> Self {
        Self::collect(&arrow.params, false, |uses| {
            uses.visit_arrow_function_body(&arrow.body)
        })
    }
    fn collect(
        parameters: &FormalParameters<'_>,
        arguments: bool,
        scan: impl FnOnce(&mut ArrayUses),
    ) -> Self {
        let mut names = Vec::new();
        for parameter in &parameters.items {
            let mut bindings = Bindings::default();
            bindings.visit_binding_pattern(&parameter.pattern);
            names.push(bindings.0);
        }
        let mut rest = Bindings::default();
        if let Some(parameter) = &parameters.rest {
            rest.visit_binding_pattern(&parameter.rest.argument);
        }
        let mut uses = ArrayUses {
            local_callback: true,
            ..Default::default()
        };
        uses.candidates.extend(names.iter().flatten().cloned());
        uses.candidates.extend(rest.0.iter().cloned());
        if arguments {
            uses.candidates.insert("arguments".to_owned());
        }
        scan(&mut uses);
        let unstable = uses.instability();
        Self {
            positions: names
                .iter()
                .enumerate()
                .filter_map(|(index, names)| {
                    names
                        .iter()
                        .any(|name| unstable.contains(name))
                        .then_some(index)
                })
                .collect(),
            rest: rest.0.iter().any(|name| unstable.contains(name)),
            arguments: arguments && unstable.contains("arguments"),
        }
    }
    pub(super) fn mutates(&self, position: usize) -> bool {
        self.positions.contains(&position) || self.rest || self.arguments
    }
}
