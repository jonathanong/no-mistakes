use super::{Evaluator, Expr, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn name(
        &mut self,
        path: &Path,
        name: &str,
        depth: u8,
        generic: bool,
    ) -> Value {
        let files = self.files;
        let file = &files[path];
        if file.facts.globals.contains_key(name) {
            if matches!(file.facts.globals.get(name), Some(Expr::Unknown)) {
                return Value::Unknown;
            }
            let env = self.module_environment(path);
            return self.scopes[env]
                .get(name)
                .cloned()
                .unwrap_or(Value::Unknown);
        }
        let import = file
            .module
            .imports
            .get(name)
            .map(|index| &file.module.ts.imported_bindings[*index]);
        let Some(import) = import else {
            return Value::Unknown;
        };
        let Some(target) = file.module.target(&import.specifier, path, &self.resolve) else {
            return Value::Unknown;
        };
        self.lookup_export(target, &import.imported, depth, generic)
            .unwrap_or(Value::Unknown)
    }
}
