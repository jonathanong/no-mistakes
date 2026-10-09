use super::{Environment, Evaluator, Function, Snapshot};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn callback_snapshot(
        &self,
        captured: Environment,
        function: &Function,
    ) -> Snapshot {
        self.callback_view()
            .snapshot(Path::new(""), captured, function, &FxHashMap::default())
            .0
    }
}
