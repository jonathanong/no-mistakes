impl Visitor<'_, '_> {
fn suppress_query(&mut self, call: &oxc_ast::ast::CallExpression<'_>, index: usize) {
    let Some(argument) = call
        .arguments
        .get(index)
        .and_then(|argument| argument.as_expression())
    else {
        return;
    };
    let Expression::ObjectExpression(object) = unwrap_ts_wrappers(argument) else {
        return;
    };
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if crate::ast::property_key_name(&property.key).as_deref() != Some("query") {
            continue;
        }
        if is_choice(&property.value, self.syntax) {
            mark_uses(&property.value, self.syntax);
            continue;
        }
        let Expression::Identifier(id) = unwrap_ts_wrappers(&property.value) else {
            continue;
        };
        let at = span(id.span);
        if self.syntax.targets.contains(&at) || self.syntax.typed_targets.contains(&at) {
            self.syntax.choice_uses.insert(at);
        }
    }
}

fn suppress_carried(&mut self, call: &oxc_ast::ast::CallExpression<'_>) {
    for index in 1..call.arguments.len() {
        self.suppress_query(call, index);
    }
}

fn direct_executor(&self, id: usize) -> bool {
    self.selections.iter().any(|selection| {
        selection.selected.contains(&id)
            || (selection.typed.contains(&id) && !selection.unsupported.contains(&id))
    })
}

fn suppress_choice(
    &mut self,
    call: &oxc_ast::ast::CallExpression<'_>,
    index: usize,
    param: usize,
) {
    let Some(argument) = call
        .arguments
        .get(index)
        .and_then(|argument| argument.as_expression())
    else {
        return;
    };
    if !is_choice(argument, self.syntax) {
        return;
    }
    let mut leaves = Vec::new();
    executor_spans(argument, self.syntax, &mut leaves);
    let sources = leaves
        .iter()
        .filter_map(|at| self.ids.get(at).copied())
        .collect::<Vec<_>>();
    let relevant = self.selections.iter().any(|selection| {
        sources.iter().any(|id| {
            selection.selected.contains(id) || selection.typed.contains(id)
        }) && selection.typed.contains(&param)
    });
    let blocked = self.selections.iter().any(|selection| {
        sources
            .iter()
            .any(|id| selection.selected.contains(id) || selection.typed.contains(id))
            && !selection.typed.contains(&param)
    });
    if relevant && !blocked {
        mark_uses(argument, self.syntax);
    }
}

fn finish(&mut self) {
    let candidates = std::mem::take(&mut self.candidates);
    for candidate in candidates {
        let called = self.called.contains(&candidate.binding);
        let passed = self.passed.iter().any(|(callee, arguments)| {
            let Some(params) = self.params.typed.get(callee) else {
                return false;
            };
            arguments.iter().enumerate().any(|(index, argument)| {
                *argument == Some(candidate.binding)
                    && params.iter().any(|(slot, _)| *slot == index)
            })
        });
        if !called && !passed {
            continue;
        }
        for selection in self.selections.iter_mut() {
            if candidate
                .sources
                .iter()
                .any(|id| selection.selected.contains(id) || selection.typed.contains(id))
            {
                selection.selected.insert(candidate.binding);
                // The choice was classified as an unsupported alias. A called
                // choice still has to prove the SQL at each invocation.
                selection.unsupported.remove(&candidate.binding);
            }
        }
        if let Some(binding) = self.bindings.get(&candidate.binding) {
            for usage in binding.references.iter().filter(|usage| usage.runtime) {
                self.syntax
                    .targets
                    .insert((usage.span.start, usage.span.end));
            }
        }
        for at in candidate.uses {
            self.syntax.choice_uses.insert(at);
        }
    }
}
}
