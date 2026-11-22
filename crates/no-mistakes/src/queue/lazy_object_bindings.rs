use super::extract_factory_scheduler_bind::{collect_object_bindings, lookup, ObjectBinding};
use crate::fx::{fx_map, FxHashMap};
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use std::cell::OnceCell;

type BindingsByName<'a> = FxHashMap<&'a str, Vec<ObjectBinding<'a>>>;

pub(super) struct LazyObjectBindings<'a> {
    pub(super) program: &'a Program<'a>,
    allocator: &'a Allocator,
    bindings: OnceCell<BindingsByName<'a>>,
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
        lookup(self.named(name), name, at)
    }

    pub(super) fn named(&self, name: &str) -> &[ObjectBinding<'a>] {
        self.bindings
            .get_or_init(|| {
                let mut by_name: BindingsByName<'_> = fx_map();
                for binding in collect_object_bindings(self.program, self.allocator) {
                    by_name.entry(binding.name).or_default().push(binding);
                }
                by_name
            })
            .get(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    pub(super) fn was_collected(&self) -> bool {
        self.bindings.get().is_some()
    }
}
