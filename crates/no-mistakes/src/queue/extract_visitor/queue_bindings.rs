use super::QueueVisitor;
use crate::queue::extract_model::{LocalQueue, ProducerBinding};
use crate::queue::object_bindings::ObjectBinding;

impl QueueVisitor<'_> {
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
