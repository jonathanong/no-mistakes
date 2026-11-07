use super::*;
impl<'a> ScopeVisitor<'a> {
    pub(super) fn record_unknown_lock_helper(&mut self, call: &CallExpression<'_>, name: &str) {
        for (executor, argument) in call.arguments.iter().enumerate() {
            let Some(expression) = argument.as_expression() else {
                continue;
            };
            let Some(_) = summary::access(expression) else {
                continue;
            };
            let receiver = match unwrap_ts_wrappers(expression) {
                Expression::StaticMemberExpression(member) => &member.object,
                expression => expression,
            };
            let mut fact = EmbeddedSqlCall::default();
            super::super::transaction::annotate_receiver(
                self,
                receiver,
                call.span.start,
                &mut fact,
            );
            if !fact.executor_transaction_known || fact.transaction_id.is_none() {
                continue;
            }
            fact.lock_scope_analysis = true;
            fact.line = byte_offset_to_line(self.source, call.span.start as usize);
            fact.callee = name.to_owned();
            fact.kind = EmbeddedSqlKind::Dynamic;
            fact.execution_unconditional = self.helper_execution_unconditional();
            let arguments = call
                .arguments
                .iter()
                .map(|argument| {
                    argument.as_expression().and_then(|expression| {
                        super::super::transaction::identity(self, expression)
                    })
                })
                .collect();
            self.imported_lock_helpers.push(Invocation {
                accesses: call
                    .arguments
                    .iter()
                    .map(|argument| argument.as_expression().and_then(summary::access))
                    .collect(),
                resolve_helper: self.awaited_lock_helper_calls.contains(&call.span.start)
                    && !call.optional
                    && self.imported_names.contains(name)
                    && self.lookup(name).is_none()
                    && call
                        .arguments
                        .iter()
                        .all(|argument| argument.as_expression().is_some()),
                name: name.to_owned(),
                executor,
                arguments,
                call: fact,
                span: (call.span.start, call.span.end),
            });
        }
    }
}

impl ScopeVisitor<'_> {
    pub(super) fn helper_execution_unconditional(&self) -> bool {
        let owner = self
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        self.control_depth == 0 && self.loop_depth == 0 && !self.exited_functions.contains(&owner)
    }
}
