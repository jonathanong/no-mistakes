use super::QueueVisitor;
use crate::queue::extract_factory::note_self_dedup;
use crate::queue::extract_record::{record_enqueue, record_flow};
use oxc_ast::ast::{CallExpression, Expression};

impl<'a> QueueVisitor<'a> {
    pub(super) fn note_enqueue_call(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = &call.callee {
            let method = member.property.name.as_str();
            if let Expression::Identifier(object) = &member.object {
                if method == "add" && self.flow_bindings.contains(object.name.as_str()) {
                    let recorded = self.facts.producers.len();
                    record_flow(
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                    self.stamp_worker(recorded);
                } else if method == "add" || method == "addBulk" {
                    let recorded = self.facts.producers.len();
                    record_enqueue(
                        object.name.as_str(),
                        method,
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                    self.stamp_worker(recorded);
                    note_self_dedup(self, call, recorded);
                }
            } else if method == "add" || method == "addBulk" {
                if let Some(binding) = self.imported_member_binding(&member.object) {
                    let recorded = self.facts.producers.len();
                    record_enqueue(
                        &binding,
                        method,
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                    self.stamp_worker(recorded);
                    note_self_dedup(self, call, recorded);
                }
            }
        }
    }
}
