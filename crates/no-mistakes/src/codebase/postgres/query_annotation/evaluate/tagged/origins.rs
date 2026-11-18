use super::*;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn record_builder_tag(&mut self, builder: u64, tag: &str, path: &Path) {
        if let Some(index) = self.fragment_tag_index(tag, path) {
            let owner = self.module_ids().id(path);
            self.builder_tags.insert(builder, (owner, index));
        }
    }

    pub(super) fn leading_fragment_prefix(
        &self,
        tag: &str,
        path: &Path,
        value: Option<&mut Value>,
    ) -> Value {
        let Some(value) = value else {
            return Value::Unknown;
        };
        self.refresh_arguments(std::slice::from_mut(value));
        match value {
            Value::Prefix(text, _, Some(builder))
                if self.same_fragment_tag(*builder, tag, path) =>
            {
                Value::Prefix(text.clone(), false, None)
            }
            _ => Value::Unknown,
        }
    }

    fn fragment_tag_index(&self, tag: &str, path: &Path) -> Option<usize> {
        let file = &self.files[path];
        if file.facts.precise_records || tag == "String.raw" {
            return None;
        }
        let index = *file.module.imports.get(tag)?;
        (file.module.ts.imported_bindings[index].specifier != "sql-template-strings")
            .then_some(index)
    }

    fn same_fragment_tag(&self, builder: u64, tag: &str, path: &Path) -> bool {
        let Some(&(owner, source_index)) = self.builder_tags.get(&builder) else {
            return false;
        };
        let Some(target_index) = self.fragment_tag_index(tag, path) else {
            return false;
        };
        let source_path = self.module_ids().path(owner);
        let source_module = &self.files[source_path].module;
        let target_module = &self.files[path].module;
        let source = &source_module.ts.imported_bindings[source_index];
        let target = &target_module.ts.imported_bindings[target_index];
        if source.imported != target.imported {
            return false;
        }
        matches!(
            (
                source_module.target(&source.specifier, source_path, &self.resolve),
                target_module.target(&target.specifier, path, &self.resolve),
            ),
            (Some(source), Some(target)) if source == target
        )
    }
}
