use super::*;

impl Scope {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn contains_builders(
        &self,
        ids: &FxHashSet<u64>,
    ) -> bool {
        let builders = self.0.builders.get_or_init(|| {
            fn collect(
                value: &Value,
                ids: &mut FxHashSet<u64>,
                visited: &mut FxHashSet<*const ()>,
            ) {
                match value {
                    Value::Prefix(_, _, Some(id)) => {
                        ids.insert(*id);
                    }
                    Value::Promise(value) | Value::Evaluated(value, _) => {
                        collect(value, ids, visited)
                    }
                    Value::Aggregate(values)
                    | Value::References(values)
                    | Value::Object(values)
                    | Value::Record(values, _)
                    | Value::Joined(values)
                    | Value::Possible(values)
                        if values.contains_reference() && visited.insert(values.identity()) =>
                    {
                        for value in values {
                            collect(value, ids, visited);
                        }
                    }
                    _ => {}
                }
            }
            let mut builders = FxHashSet::default();
            let mut visited = FxHashSet::default();
            for value in self.values() {
                collect(value, &mut builders, &mut visited);
            }
            builders
        });
        !builders.is_disjoint(ids)
    }
}
