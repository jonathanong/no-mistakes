use super::*;
impl<F: Fn(&str, &std::path::Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn replace_builder(
        &mut self,
        replacement: &Value,
    ) {
        fn identity(value: &Value) -> Option<u64> {
            match value {
                Value::Prefix(_, _, Some(id)) => Some(*id),
                Value::Joined(values) if !values.is_empty() => {
                    let id = identity(&values[0])?;
                    values
                        .iter()
                        .all(|value| identity(value) == Some(id))
                        .then_some(id)
                }
                _ => None,
            }
        }
        let Some(id) = identity(replacement) else {
            return;
        };
        self.write_builder(id, replacement.clone());
        fn replace(value: &mut Value, id: u64, replacement: &Value) {
            if matches!(value, Value::Prefix(_, _, Some(other)) if *other == id) {
                *value = replacement.clone();
            } else {
                match value {
                    Value::Promise(value) | Value::Evaluated(value, _) => {
                        replace(value, id, replacement)
                    }
                    Value::Aggregate(values)
                    | Value::References(values)
                    | Value::Object(values)
                    | Value::Record(values, _)
                    | Value::Joined(values)
                    | Value::Possible(values) => {
                        for value in values {
                            replace(value, id, replacement);
                        }
                    }
                    _ => {}
                }
            }
        }
        let ids = FxHashSet::from_iter([id]);
        for cell in self.builder_locations(&ids) {
            if let super::super::transactions::Saved::Value(Some(mut value)) = self.read_cell(&cell)
            {
                replace(&mut value, id, replacement);
                self.apply_cell(
                    &cell,
                    &super::super::transactions::Saved::Value(Some(value)),
                );
            }
        }
    }
}
