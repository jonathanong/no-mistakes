use super::QueueVisitor;
use crate::queue::extract_factory::note_self_dedup;
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_record::{record_enqueue, record_flow};
use oxc_ast::ast::{CallExpression, Expression};

impl<'a> QueueVisitor<'a> {
    pub(super) fn note_enqueue_call(&mut self, call: &CallExpression<'a>) {
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let method = member.property.name.as_str();
        if !matches!(method, "add" | "addBulk") {
            return;
        }
        let binding = if let Expression::Identifier(object) = &member.object {
            self.producer_binding(object.name.as_str(), call.span.start)
        } else if let Some(binding) = self.imported_member_binding(&member.object) {
            let Some((namespace, _)) = binding.split_once('.') else {
                return;
            };
            if matches!(
                self.producer_binding(namespace, call.span.start),
                ProducerBinding::Local(_)
            ) {
                return;
            }
            ProducerBinding::Named(binding)
        } else {
            return;
        };
        let recorded = self.facts.producers.len();
        if method == "add"
            && matches!(binding, ProducerBinding::Local(at) if self.flow_bindings.contains(&at))
        {
            record_flow(
                self.path,
                self.source,
                &self.const_strings,
                &mut self.facts,
                call,
            );
        } else {
            record_enqueue(
                &binding,
                method,
                self.path,
                self.source,
                &self.const_strings,
                &mut self.facts,
                call,
            );
            if method == "addBulk" {
                crate::queue::extract_debounce::check_bulk_call(self, call);
            } else {
                note_self_dedup(self, call, recorded);
            }
        }
        self.stamp_worker(recorded);
    }
}
