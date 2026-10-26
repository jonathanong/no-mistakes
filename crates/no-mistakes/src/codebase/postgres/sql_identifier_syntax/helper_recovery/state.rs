use super::sql_text::SqlText;
use super::value::Value;
use crate::fx::FxHashMap;

#[derive(Clone, Default)]
pub(super) struct State {
    pub env: FxHashMap<String, Value>,
    pub heap: Vec<SqlText>,
    pub returned: Option<Value>,
}
impl State {
    pub fn bytes(&self) -> usize {
        self.heap
            .iter()
            .map(|text| text.raw.sql.len())
            .sum::<usize>()
            + self.env.values().map(Value::bytes).sum::<usize>()
    }
    pub fn allocate(&mut self, sql: SqlText) -> Value {
        let id = self.heap.len();
        self.heap.push(sql);
        Value::Builder(id)
    }
    pub fn contains_executor(&self, value: &Value) -> bool {
        match value {
            Value::Builder(id) => self.heap.get(*id).is_some_and(|text| text.executor_payload),
            Value::Object(props) => props.iter().any(|(_, value)| self.contains_executor(value)),
            Value::Array(items) => items.iter().any(|value| self.contains_executor(value)),
            _ => value.contains_executor(),
        }
    }
    pub fn materialize(&self, value: Value) -> Option<Value> {
        match value {
            Value::Builder(id) => Some(Value::Sql(self.heap.get(id)?.clone())),
            other if other.read_only_argument() => Some(other),
            _ => None,
        }
    }
    pub fn text(&self, value: &Value) -> Option<SqlText> {
        match value {
            Value::Text(text) | Value::Sql(text) => Some(text.clone()),
            Value::Builder(id) => self.heap.get(*id).cloned(),
            _ => None,
        }
    }
}
