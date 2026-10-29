use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
use crate::codebase::postgres::executor_detachment::{executor_bindings, executor_receiver};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, FormalParameter, IdentifierReference};
use oxc_span::{GetSpan, Span};

impl ScopeVisitor<'_> {
    pub(super) fn note_executor_member(&mut self, expression: &Expression<'_>) {
        if executor_receiver(expression, &self.executor_options.executor_names)
            .is_some_and(|receiver| self.detached_query_receiver(receiver))
        {
            self.unresolved_executor_reference(unwrap_ts_wrappers(expression).span());
        }
    }

    pub(super) fn note_executor_identifier(&mut self, reference: &IdentifierReference<'_>) {
        let name = reference.name.as_str();
        let Some(scope) = self.capture_scope(name) else {
            return;
        };
        let names = self.executor_alias_refs.entry(scope).or_default();
        if let Some(uses) = names.get_mut(name) {
            uses.push(reference.span);
        } else {
            names.insert(name.to_owned(), vec![reference.span]);
        }
    }

    pub(super) fn finish_executor_references(&mut self) {
        for (scope, names) in std::mem::take(&mut self.executor_alias_refs) {
            for (name, spans) in names {
                if self.has_capture_key(scope, &name) {
                    for span in spans {
                        self.unresolved_executor_reference(span);
                    }
                }
            }
        }
    }

    pub(super) fn note_executor_parameter(&mut self, parameter: &FormalParameter<'_>) {
        let (pattern, receiver) = match (&parameter.pattern, &parameter.initializer) {
            (pattern, Some(receiver)) => (pattern, receiver.as_ref()),
            (BindingPattern::AssignmentPattern(default), _) => (&default.left, &default.right),
            _ => return,
        };
        if let BindingPattern::BindingIdentifier(local) = pattern {
            self.bind_receiver_alias(local.name.as_str(), receiver);
            self.initialize_detached_value(local.name.as_str(), receiver, false);
        }
        if !self.detached_query_receiver(receiver) {
            return;
        }
        for (local, _) in executor_bindings(pattern, &self.executor_options.executor_names) {
            self.note_query_capture_with_certainty(local.name.as_str(), receiver, false);
            self.unresolved_executor_reference(local.span);
        }
    }

    fn unresolved_executor_reference(&mut self, span: Span) {
        let key = (span.start, span.end);
        if self.executor_direct_refs.contains(&key) || !self.executor_reference_spans.insert(key) {
            return;
        }
        self.calls.push(EmbeddedSqlCall {
            line: crate::codebase::ts_source::byte_offset_to_line(self.source, span.start as usize),
            callee: self.source[span.start as usize..span.end as usize].to_owned(),
            kind: EmbeddedSqlKind::Dynamic,
            ..Default::default()
        });
        self.call_spans.push((span.start, span.end));
        if self.track_order {
            self.confirmed_order.push(self.next_seq);
        }
        self.next_seq += 1;
    }
}
