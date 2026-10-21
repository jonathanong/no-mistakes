use super::super::{Environment, Evaluator, Function, Value};
use super::state::Snapshot;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn imported_callback_snapshot(
        &mut self,
        path: &Path,
        captured: Environment,
        function: &Function,
        depth: u8,
    ) -> Snapshot {
        let module_ids = std::sync::Arc::clone(self.shared_module_ids());
        let mut imports: FxHashMap<
            super::super::super::module_ids::ModuleId,
            FxHashMap<String, Value>,
        > = FxHashMap::default();
        loop {
            let (snapshot, missing) = self
                .callback_view()
                .snapshot(path, captured, function, &imports);
            if missing.is_empty() {
                return snapshot;
            }
            let mut missing = missing.into_iter().collect::<Vec<_>>();
            missing.sort_unstable();
            for (module_id, name) in missing {
                let value = self.name(module_ids.path(module_id), &name, depth, false);
                imports.entry(module_id).or_default().insert(name, value);
            }
        }
    }
}
