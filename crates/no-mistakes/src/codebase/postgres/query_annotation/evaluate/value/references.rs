use super::*;

impl Values {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn contains_environment(
        &self,
    ) -> bool {
        !self.environment_indices().is_empty()
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn environment_indices(
        &self,
    ) -> &FxHashSet<Environment> {
        self.0.environments.get_or_init(|| {
            fn collect(value: &Value, environments: &mut FxHashSet<Environment>) {
                match value {
                    Value::Function(_, _, env) => {
                        environments.insert(*env);
                    }
                    Value::Promise(value) | Value::Evaluated(value, _) => {
                        collect(value, environments)
                    }
                    Value::Aggregate(values)
                    | Value::References(values)
                    | Value::Object(values)
                    | Value::Record(values, _)
                    | Value::Joined(values)
                    | Value::Possible(values) => {
                        environments.extend(values.environment_indices());
                    }
                    _ => {}
                }
            }
            let mut environments = FxHashSet::default();
            for value in &self.0.values {
                collect(value, &mut environments);
            }
            environments
        })
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn contains_arguments(
        &self,
    ) -> bool {
        *self
            .0
            .contains_arguments
            .get_or_init(|| self.0.values.iter().any(Value::contains_arguments))
    }
}

impl Value {
    fn contains_arguments(&self) -> bool {
        match self {
            Self::Arguments(_) | Self::Array(_) => true,
            Self::Promise(value) | Self::Evaluated(value, _) => value.contains_arguments(),
            Self::Aggregate(values)
            | Self::References(values)
            | Self::Object(values)
            | Self::Record(values, _)
            | Self::Joined(values)
            | Self::Possible(values) => values.contains_arguments(),
            _ => false,
        }
    }
}
