use super::*;

impl Collector<'_> {
    pub(super) fn form(
        &self,
        expression: &Expression<'_>,
        profile: usize,
    ) -> Option<(usize, Form, bool, Vec<Span>)> {
        let receiver = executor_receiver(expression, &self.profiles[profile].executor_names);
        if let Some((root, parts)) = chain(receiver.unwrap_or(expression)) {
            if parts.len() != 1 {
                return None;
            }
            let form = if receiver.is_some() {
                Form::Query
            } else {
                Form::Copy
            };
            return Some((*self.owners.get(&root)?, form, true, vec![root]));
        }
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        let method = bind_method(call)?;
        let receiver = executor_receiver(method, &self.profiles[profile].executor_names);
        let (root, parts) = chain(receiver.unwrap_or(method))?;
        if parts.len() != 1 {
            return None;
        }
        let query = receiver.is_some();
        let mut uses = vec![root];
        let receiver = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .and_then(chain)
            .filter(|(_, parts)| parts.len() == 1)
            .and_then(|(at, _)| {
                uses.push(at);
                self.owners.get(&at).copied()
            });
        Some((
            *self.owners.get(&root)?,
            Form::Bind(receiver, query),
            call.arguments.len() == 1 && !call.optional,
            uses,
        ))
    }
    pub(super) fn pattern(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
        immutable: bool,
    ) {
        for profile in 0..self.profiles.len() {
            let Some((source, form, exact, uses)) = self.form(init, profile) else {
                continue;
            };
            match pattern {
                BindingPattern::BindingIdentifier(alias) => {
                    self.add(alias, profile, source, form, exact && immutable, uses)
                }
                BindingPattern::ObjectPattern(_) if matches!(form, Form::Copy) => {
                    for (alias, plain) in
                        executor_bindings(pattern, &self.profiles[profile].executor_names)
                    {
                        self.add(
                            alias,
                            profile,
                            source,
                            Form::Query,
                            exact && immutable && plain,
                            uses.clone(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
