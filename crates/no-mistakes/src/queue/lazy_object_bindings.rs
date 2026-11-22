use super::extract_factory_scheduler_bind::{collect_object_bindings, lookup, ObjectBinding};
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use std::cell::OnceCell;

pub(super) struct LazyObjectBindings<'a> {
    pub(super) program: &'a Program<'a>,
    allocator: &'a Allocator,
    bindings: OnceCell<Vec<ObjectBinding<'a>>>,
}

impl<'a> LazyObjectBindings<'a> {
    pub(super) fn new(program: &'a Program<'a>, allocator: &'a Allocator) -> Self {
        Self {
            program,
            allocator,
            bindings: OnceCell::new(),
        }
    }

    pub(super) fn lookup(&self, name: &str, at: u32) -> Option<&ObjectBinding<'a>> {
        lookup(self.all(), name, at)
    }

    fn all(&self) -> &[ObjectBinding<'a>] {
        self.bindings
            .get_or_init(|| collect_object_bindings(self.program, self.allocator))
    }

    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    pub(super) fn was_collected(&self) -> bool {
        self.bindings.get().is_some()
    }
}
