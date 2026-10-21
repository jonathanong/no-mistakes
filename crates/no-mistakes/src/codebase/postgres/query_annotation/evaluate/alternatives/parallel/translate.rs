use super::super::super::value::Values;
use super::*;
use crate::fx::FxHashMap;
/// Translate full allocated ranges, retaining original DAG keys until completion.
pub(super) struct Translation {
    pub base_env: usize,
    pub base_id: u64,
    pub env_offset: usize,
    pub id_offset: u64,
    values: FxHashMap<usize, (Values, Option<Values>)>,
}
impl Translation {
    pub fn new(base_env: usize, base_id: u64, env: usize, id: u64) -> Self {
        Self {
            base_env,
            base_id,
            env_offset: env - base_env,
            id_offset: id - base_id,
            values: Default::default(),
        }
    }
    pub fn identity(&self) -> bool {
        self.env_offset == 0 && self.id_offset == 0
    }
    pub fn env(&self, env: usize) -> usize {
        env + if env >= self.base_env {
            self.env_offset
        } else {
            0
        }
    }
    pub fn id(&self, id: u64) -> u64 {
        id + if id >= self.base_id {
            self.id_offset
        } else {
            0
        }
    }
    pub fn value(&mut self, value: Value) -> Value {
        self.changed(&value).unwrap_or(value)
    }
    pub fn changed(&mut self, value: &Value) -> Option<Value> {
        if self.identity() {
            return None;
        }
        Some(match value {
            Value::Arguments(id) if self.id(*id) != *id => Value::Arguments(self.id(*id)),
            Value::Array(id) if self.id(*id) != *id => Value::Array(self.id(*id)),
            Value::Prefix(text, safe, Some(id)) if self.id(*id) != *id => {
                Value::Prefix(text.clone(), *safe, Some(self.id(*id)))
            }
            Value::Function(function, path, env) if self.env(*env) != *env => {
                Value::Function(function.clone(), path.clone(), self.env(*env))
            }
            Value::Promise(value) => Value::Promise(Box::new(self.changed(value)?)),
            Value::Evaluated(value, handled) => {
                Value::Evaluated(Box::new(self.changed(value)?), *handled)
            }
            Value::Aggregate(values) => Value::Aggregate(self.values(values)?),
            Value::References(values) => Value::References(self.values(values)?),
            Value::Object(values) => Value::Object(self.values(values)?),
            Value::Record(values, names) => Value::Record(self.values(values)?, names.clone()),
            Value::Joined(values) => Value::Joined(self.values(values)?),
            Value::Possible(values) => Value::Possible(self.values(values)?),
            _ => return None,
        })
    }
    fn values(&mut self, values: &Values) -> Option<Values> {
        let key = values.identity() as usize;
        if let Some((_, translated)) = self.values.get(&key) {
            return translated.clone();
        }
        if !values.contains_reference()
            && !values
                .environment_indices()
                .iter()
                .any(|env| self.env(*env) != *env)
        {
            return None;
        }
        let mut translated: Option<Vec<Value>> = None;
        for (index, value) in values.iter().enumerate() {
            if let Some(changed) = self.changed(value) {
                translated
                    .get_or_insert_with(|| values[..index].to_vec())
                    .push(changed);
            } else if let Some(output) = &mut translated {
                output.push(value.clone());
            }
        }
        let translated = translated.map(Values::from);
        self.values
            .insert(key, (values.clone(), translated.clone()));
        translated
    }
}
