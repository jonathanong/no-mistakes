use super::*;

const ALIAS_DEPTH: usize = 8;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn contextual_parameter(
        &self,
        path: &std::path::Path,
        callback: &ValueType,
        index: usize,
        visited: &mut FxHashSet<(std::path::PathBuf, String)>,
    ) -> FlowValue {
        match self
            .signature(path, callback, 0)
            .and_then(|signature| signature.into_iter().nth(index))
        {
            Some(shape) => self.annotation_value(path, &shape, None, visited),
            None => FlowValue::Unknown,
        }
    }

    fn signature(
        &self,
        path: &std::path::Path,
        shape: &ValueType,
        depth: usize,
    ) -> Option<Vec<ValueType>> {
        match shape {
            ValueType::Signature(parameters) => Some(parameters.clone()),
            ValueType::Member(base, name) => {
                let properties = self.properties(path, base, depth)?;
                self.signature(path, properties.get(name)?, depth)
            }
            _ => None,
        }
    }

    fn properties(
        &self,
        path: &std::path::Path,
        shape: &ValueType,
        depth: usize,
    ) -> Option<FxHashMap<String, ValueType>> {
        match shape {
            ValueType::Object(properties) => Some(properties.clone()),
            ValueType::Reference(name) if depth < ALIAS_DEPTH => {
                let alias = self
                    .facts
                    .get_ts_facts(path)?
                    .value_flow
                    .type_shapes
                    .get(name)?;
                self.properties(path, alias, depth + 1)
            }
            _ => None,
        }
    }
}
