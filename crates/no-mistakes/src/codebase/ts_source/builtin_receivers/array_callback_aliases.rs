use super::*;

#[path = "array_callback_parameters.rs"]
mod parameter_uses;
use parameter_uses::Parameters;
#[derive(Default)]
struct Callbacks {
    declarations: FxHashMap<String, usize>,
    requested: FxHashSet<String>,
    parameters: FxHashMap<String, Parameters>,
}
impl<'a> Visit<'a> for Callbacks {
    fn visit_binding_identifier(&mut self, id: &BindingIdentifier<'a>) {
        *self.declarations.entry(id.name.to_string()).or_default() += 1;
    }
    fn visit_variable_declarator(&mut self, node: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(id), Some(init)) = (&node.id, &node.init) {
            let parameters = match init.without_parentheses() {
                _ if !self.requested.contains(id.name.as_str()) => None,
                Expression::ArrowFunctionExpression(arrow) => Some(Parameters::arrow(arrow)),
                Expression::FunctionExpression(function) => Some(Parameters::function(function)),
                _ => None,
            };
            if let Some(parameters) = parameters {
                self.parameters.insert(id.name.to_string(), parameters);
            }
        }
        walk::walk_variable_declarator(self, node);
    }
    fn visit_function(&mut self, node: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let Some(id) = &node.id {
            if self.requested.contains(id.name.as_str())
                && !self.parameters.contains_key(id.name.as_str())
            {
                self.parameters
                    .insert(id.name.to_string(), Parameters::function(node));
            }
        }
        walk::walk_function(self, node, flags);
    }
}
struct Aliases<'n> {
    names: &'n Names,
    callbacks: &'n Callbacks,
    uses: &'n mut ArrayUses,
}
impl<'a> Visit<'a> for Aliases<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() {
            let method = member.property.name.as_str();
            let receiver_parameter = match method {
                "reduce" | "reduceRight" => Some(3),
                "forEach" | "map" | "filter" | "flatMap" | "some" | "every" | "find"
                | "findIndex" | "findLast" | "findLastIndex" => Some(2),
                _ => None,
            };
            if let (Some(position), Some(source)) =
                (receiver_parameter, source_array(&member.object))
            {
                let inert =
                    matches!(
                        self.names.value_kind(&member.object, 0),
                        Some(ValueKind::Array {
                            length: Some(0),
                            ..
                        })
                    ) || (!matches!(method, "find" | "findIndex" | "findLast" | "findLastIndex")
                        && matches!(
                            self.names.value_kind(&member.object, 0),
                            Some(ValueKind::Array { empty: true, .. })
                        ));
                if self.uses.candidates.contains(source) && !inert {
                    let parameters = call
                        .arguments
                        .first()
                        .and_then(Argument::as_expression)
                        .and_then(|callback| match callback.without_parentheses() {
                            Expression::ArrowFunctionExpression(arrow) => {
                                Some(Parameters::arrow(arrow))
                            }
                            Expression::FunctionExpression(function) => {
                                Some(Parameters::function(function))
                            }
                            Expression::Identifier(id)
                                if self.callbacks.declarations.get(id.name.as_str())
                                    == Some(&1) =>
                            {
                                self.callbacks.parameters.get(id.name.as_str()).cloned()
                            }
                            _ => None,
                        });
                    if parameters.is_some_and(|parameters| parameters.mutates(position)) {
                        self.uses.forced.insert(source.to_owned());
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}
pub(super) fn collect(names: &Names, program: &Program<'_>, uses: &mut ArrayUses) {
    let mut callbacks = Callbacks {
        requested: std::mem::take(&mut uses.callback_names),
        ..Default::default()
    };
    callbacks.visit_program(program);
    #[cfg(feature = "test-instrumentation")]
    LAST_SUMMARIES.set(callbacks.parameters.len());
    Aliases {
        names,
        callbacks: &callbacks,
        uses,
    }
    .visit_program(program);
}

#[cfg(feature = "test-instrumentation")]
thread_local! { static LAST_SUMMARIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
#[path = "array_callback_aliases_tests.rs"]
pub(crate) mod tests;

#[cfg(feature = "test-instrumentation")]
pub(super) fn reset_summaries() {
    LAST_SUMMARIES.set(0);
}
