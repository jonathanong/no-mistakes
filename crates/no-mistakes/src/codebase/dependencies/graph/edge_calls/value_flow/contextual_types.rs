use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn contextual_parameter_origin(
        &self,
        path: &std::path::Path,
        parameter: &ValueParameter,
    ) -> Option<(String, String)> {
        let inputs = self.incoming.get(&(path.to_path_buf(), parameter.owner))?;
        let mut agreed = None;
        for input in inputs {
            if input.synthetic_callback {
                continue;
            }
            if input.callback {
                return None;
            }
            let index = self.indexes.file(self.facts, &input.file)?;
            let call = index
                .values
                .sites
                .get(&input.offset)
                .and_then(|sites| sites.get(&input.callee))
                .and_then(|key| index.values.calls.get(key))?;
            let ValueExpression::Named { name, scope, .. } = &call.callee else {
                return None;
            };
            let callee = scoped_name(&index.values.parameters, (*scope)?, name)?;
            if callee.invalidated_at.is_some() {
                return None;
            }
            let source = self
                .facts
                .get_ts_facts(&input.file)?
                .value_flow
                .type_sources
                .as_ref()?
                .parameters
                .get(&(callee.owner, callee.index))?;
            let origin = self.contextual_annotation_origin(&input.file, source, parameter.index)?;
            if agreed.as_ref().is_some_and(|previous| *previous != origin) {
                return None;
            }
            agreed = Some(origin);
        }
        agreed
    }
}
