use super::Value;
use crate::fx::FxHashSet;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum Root {
    Argument(u64),
    Container(*const ()),
}

fn root(mut value: &Value) -> Option<Root> {
    loop {
        match value {
            Value::Arguments(id) | Value::Array(id) => return Some(Root::Argument(*id)),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => return Some(Root::Container(values.identity())),
            Value::Promise(inner) | Value::Evaluated(inner, _) => value = inner,
            _ => return None,
        }
    }
}

// Borrowed scopes and both arenas stay fixed for this comparison phase.
#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) struct ComparisonMemo {
    compared: FxHashSet<(Root, Root)>,
}

impl ComparisonMemo {
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn repeated(
        &mut self,
        before: &Value,
        after: &Value,
    ) -> Option<bool> {
        Some(!self.compared.insert((root(before)?, root(after)?)))
    }
}
