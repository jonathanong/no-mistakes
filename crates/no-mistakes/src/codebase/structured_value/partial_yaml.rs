use crate::fx::FxHashSet;
use serde_yaml::{Mapping, Value};

pub(super) type UnknownMappings = FxHashSet<usize>;

#[derive(Debug)]
pub(crate) struct PartialYaml {
    // Expansion visits parents before children and stops mutating each mapping
    // before visiting its children, so mapping addresses stay stable in this box.
    value: Box<Value>,
    mappings: UnknownMappings,
}

impl PartialYaml {
    pub(super) fn new(value: Box<Value>, mappings: UnknownMappings) -> Self {
        Self { value, mappings }
    }

    pub(crate) fn value(&self) -> &Value {
        &self.value
    }

    pub(crate) fn mapping_unknown(&self, value: &Value) -> bool {
        value
            .as_mapping()
            .is_some_and(|mapping| self.mappings.contains(&mapping_id(mapping)))
    }

    pub(crate) fn contains_unknown(&self, value: &Value) -> bool {
        if self.mappings.is_empty() {
            return false;
        }
        let mut pending = vec![value];
        while let Some(value) = pending.pop() {
            match value {
                Value::Mapping(mapping) => {
                    if self.mapping_unknown(value) {
                        return true;
                    }
                    pending.extend(mapping.values());
                }
                Value::Sequence(items) => pending.extend(items),
                Value::Tagged(tagged) => pending.push(&tagged.value),
                _ => {}
            }
        }
        false
    }
}

pub(super) fn mapping_id(mapping: &Mapping) -> usize {
    std::ptr::from_ref(mapping) as usize
}
