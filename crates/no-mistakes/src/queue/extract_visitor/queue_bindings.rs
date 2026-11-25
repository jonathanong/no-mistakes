use super::QueueVisitor;
use crate::queue::extract_model::{LocalQueue, ProducerBinding};
use crate::queue::object_bindings::ObjectBinding;
use oxc_ast::ast::{Argument, Expression};

impl QueueVisitor<'_> {
    pub(in crate::queue) fn resolve_queue_name_arg(
        &mut self,
        arg: &Argument<'_>,
    ) -> (Option<String>, Option<String>) {
        if let Some(value) = self.literal_arg(arg) {
            return (Some(value), None);
        }
        if let Some(ident) = crate::queue::extract_factory::identifier_arg(arg) {
            if self.facts.string_imports.contains_key(&ident) {
                return (None, Some(ident));
            }
        }
        let Some(Expression::StaticMemberExpression(member)) =
            arg.as_expression().map(Expression::get_inner_expression)
        else {
            return (None, None);
        };
        let Expression::Identifier(object) = member.object.get_inner_expression() else {
            return (None, None);
        };
        if member.property.name == "name" {
            match self.local_queue(object.name.as_str(), object.span.start) {
                Some(LocalQueue::Known(queue_name)) => return (Some(queue_name), None),
                Some(LocalQueue::ImportedName(imported) | LocalQueue::ImportedQueue(imported)) => {
                    return (None, Some(imported));
                }
                None => {}
            }
        }
        if let Some(specifier) = self.namespace_imports.get(object.name.as_str()).cloned() {
            let member_name = member.property.name.as_str();
            let binding = format!("{}.{member_name}", object.name);
            self.facts
                .string_imports
                .insert(binding.clone(), (specifier, member_name.to_string()));
            return (None, Some(binding));
        }
        (None, None)
    }
    pub fn producer_binding(&self, name: &str, at: u32) -> ProducerBinding {
        self.lexical_binding(name, at)
            .map(|binding| ProducerBinding::Local(binding.resolve_at))
            .unwrap_or_else(|| ProducerBinding::Named(name.to_string()))
    }

    pub(super) fn lexical_binding(&self, name: &str, at: u32) -> Option<&ObjectBinding<'_>> {
        ObjectBinding::in_scope(self.object_bindings.named(name), name, at)
    }

    pub fn local_queue(&self, name: &str, at: u32) -> Option<LocalQueue> {
        if let Some(binding) = self.lexical_binding(name, at) {
            return self
                .facts
                .queue_declarations
                .get(&binding.resolve_at)
                .cloned();
        }
        self.facts
            .string_imports
            .contains_key(name)
            .then(|| LocalQueue::ImportedQueue(name.to_string()))
    }

    pub fn note_queue_declaration(&mut self, name: &str, declaration: u32, queue: LocalQueue) {
        let module_binding = self.object_bindings.named(name).iter().any(|binding| {
            binding.name == name && binding.resolve_at == declaration && binding.scope_depth == 0
        });
        if module_binding {
            match &queue {
                LocalQueue::Known(queue_name) => {
                    self.facts
                        .queue_bindings
                        .insert(name.to_string(), queue_name.clone());
                    self.facts
                        .queue_exports
                        .insert(name.to_string(), queue_name.clone());
                }
                LocalQueue::ImportedName(imported) => {
                    self.facts
                        .pending_queue_bindings
                        .insert(name.to_string(), imported.clone());
                }
                LocalQueue::ImportedQueue(local) => {
                    self.facts.reexports.push((name.to_string(), local.clone()));
                }
            }
        }
        self.facts.queue_declarations.insert(declaration, queue);
    }
}
