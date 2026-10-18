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
        fn needs_replacement(value: &Value, id: u64, replacement: &Value) -> bool {
            match value {
                Value::Prefix(_, _, Some(other)) if *other == id => value != replacement,
                Value::Promise(value) | Value::Evaluated(value, _) => {
                    needs_replacement(value, id, replacement)
                }
                Value::Aggregate(values)
                | Value::References(values)
                | Value::Object(values)
                | Value::Record(values, _)
                | Value::Joined(values)
                | Value::Possible(values) => values
                    .iter()
                    .any(|value| needs_replacement(value, id, replacement)),
                _ => false,
            }
        }
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
        let locations = self.builder_locations(&ids);
        let mut changed = false;
        for cell in &locations {
            if let super::super::transactions::Saved::Value(Some(mut value)) = self.read_cell(cell)
            {
                if !needs_replacement(&value, id, replacement) {
                    continue;
                }
                changed = true;
                replace(&mut value, id, replacement);
                self.apply_cell(cell, &super::super::transactions::Saved::Value(Some(value)));
            }
        }
        // An unchanged append has no new canonical state. Inserting one would
        // make an untouched sibling look missing instead of preserving its SQL.
        if changed || locations.is_empty() || self.builder_updates.contains_key(&id) {
            self.write_builder(id, replacement.clone());
        }
    }
}
