use super::{resolve, ScopeVisitor};
use crate::codebase::postgres::embedded::fragment_debug::PendingImportedSqlCall;
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::walk;
use oxc_span::GetSpan;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn visit_sql_call(&mut self, call: &CallExpression<'a>) {
        let helper_effect = self.call_builder_effect(call);
        let append_effect = self.append_alias_effect(call);
        if self.suppress_nested_builder_fragments == 0 {
            if let Some(argument) = call.arguments.first() {
                if let Some(sql_text) = resolve::appended_builder_fragment(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        Some(sql_text),
                        argument.as_expression(),
                        Some(call.span.start),
                    );
                } else if resolve::is_builder_append(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        None,
                        argument.as_expression(),
                        Some(call.span.start),
                    );
                }
            }
        }
        let variants = self
            .appended_variants(call)
            .map(|(name, values)| (name.to_string(), values));
        resolve::apply_append(self, call);
        if let Some((name, values)) = variants {
            self.set_variants(&name, values);
        }
        let before = (self.calls.len(), self.pending_calls.len());
        let imported_helper = super::super::first_call_argument(call)
            .and_then(|argument| super::imported::unresolved_helper_name(argument, self));
        resolve::record_executor_call(self, call);
        if self.calls.len() > before.0 {
            if let Some(local) = imported_helper {
                if self.calls[before.0].kind == super::EmbeddedSqlKind::Dynamic
                    && self.calls[before.0].sql_text.is_none()
                {
                    self.pending_imported_calls.push(PendingImportedSqlCall {
                        span: (call.span.start, call.span.end),
                        local,
                    });
                }
            }
        }
        let arguments = self.begin_executor_arguments(call, before);
        self.record_argument_append_effects(&append_effect, call.span.start);
        let invocation = self.begin_argument_invocation(call);
        walk::walk_call_expression(self, call);
        self.finish_argument_invocation(invocation);
        self.record_argument_helper_effects(&helper_effect.0, call.span.start);
        if let Some(arguments) = arguments {
            // Capture all argument evaluations before this invocation consumes
            // the builder. An unconfirmed relative callee can invalidate later
            // reuse without erasing its own pending executor arguments.
            self.finish_executor_arguments(call, arguments);
        }
        self.apply_call_builder_effect(helper_effect);
        self.apply_append_alias_effect(append_effect);
    }
}
