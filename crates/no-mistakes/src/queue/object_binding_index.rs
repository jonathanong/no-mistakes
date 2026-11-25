use super::ObjectBinding;
use crate::fx::FxHashMap;
use std::cell::{OnceCell, RefCell};

type BindingsByName<'a> = FxHashMap<&'a str, Vec<ObjectBinding<'a>>>;

#[derive(Default)]
pub(in crate::queue) struct ObjectBindings<'a> {
    by_name: OnceCell<BindingsByName<'a>>,
    declarations: RefCell<Vec<ObjectBinding<'a>>>,
}

impl<'a> ObjectBindings<'a> {
    pub(in crate::queue) fn lookup(&self, name: &str, at: u32) -> Option<&ObjectBinding<'a>> {
        let binding = ObjectBinding::in_scope(self.named(name), name, at)?;
        (at >= binding.visible_from).then_some(binding)
    }

    pub(in crate::queue) fn named(&self, name: &str) -> &[ObjectBinding<'a>] {
        self.by_name
            .get_or_init(|| {
                let mut by_name = BindingsByName::default();
                for binding in self.declarations.take() {
                    by_name.entry(binding.name).or_default().push(binding);
                }
                by_name
            })
            .get(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    pub(in crate::queue) fn was_indexed(&self) -> bool {
        self.by_name.get().is_some()
    }

    pub(super) fn insert(&mut self, binding: ObjectBinding<'a>) {
        self.declarations.get_mut().push(binding);
    }
}
