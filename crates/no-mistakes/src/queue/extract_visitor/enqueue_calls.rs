use super::QueueVisitor;
use crate::queue::extract_factory::note_self_dedup;
use crate::queue::extract_record::{record_enqueue, record_flow};
use oxc_ast::ast::{CallExpression, Expression};

impl<'a> QueueVisitor<'a> {
    fn binding_is_shadowed(&self, name: &str, at: u32) -> bool {
        let declaration = self.queue_declarations.get(name);
        let imported = self.facts.string_imports.contains_key(name)
            || self.namespace_imports.contains_key(name);
        if declaration.is_none() && !imported {
            return false;
        }
        let active = self
            .object_bindings
            .iter()
            .filter(|binding| {
                binding.name == name && binding.scope_start <= at && at < binding.scope_end
            })
            .max_by_key(|binding| binding.scope_start);
        if let Some(declaration) = declaration {
            return active.is_none_or(|binding| binding.resolve_at != *declaration);
        }
        active.is_some()
    }

    pub(super) fn note_enqueue_call(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = &call.callee {
            let method = member.property.name.as_str();
            if !matches!(method, "add" | "addBulk") {
                return;
            }
            if let Expression::Identifier(object) = &member.object {
                if self.binding_is_shadowed(object.name.as_str(), call.span.start) {
                    return;
                }
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
                } else {
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
            } else if let Some(binding) = self.imported_member_binding(&member.object) {
                let Some((namespace, _)) = binding.split_once('.') else {
                    return;
                };
                if self.binding_is_shadowed(namespace, call.span.start) {
                    return;
                }
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
