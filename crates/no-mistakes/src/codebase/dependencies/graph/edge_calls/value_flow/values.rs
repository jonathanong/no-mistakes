use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn value(&mut self, path: &std::path::Path, expression: &ValueExpression) -> FlowValue {
        match expression {
            ValueExpression::Named {
                name,
                scope,
                offset,
            } => self.named(path, name, *scope, *offset),
            ValueExpression::Callable(id) => self.callable(path, *id),
            ValueExpression::Member(base, member) => {
                let base = self.value(path, base);
                self.member(base, member)
            }
            ValueExpression::Object(fields) => FlowValue::Object(
                fields
                    .iter()
                    .map(|(name, value)| (name.clone(), self.value(path, value)))
                    .collect(),
            ),
            ValueExpression::Call(offset) => self.returned(path, *offset),
            ValueExpression::Unknown => FlowValue::Unknown,
        }
    }
    pub(super) fn callable(&self, path: &std::path::Path, id: CallableId) -> FlowValue {
        let Some(file) = self.indexes.file(self.facts, path) else {
            return FlowValue::Unknown;
        };
        file.values
            .scopes
            .get(&id)
            .map(|scope| FlowValue::Callable {
                file: path.to_path_buf(),
                scope: scope.clone(),
                id,
            })
            .unwrap_or(FlowValue::Unknown)
    }
    pub(super) fn member(&self, base: FlowValue, member: &str) -> FlowValue {
        match base {
            FlowValue::Object(fields) => fields
                .into_iter()
                .rev()
                .find_map(|(name, value)| (name == member).then_some(value))
                .unwrap_or(FlowValue::Unknown),
            FlowValue::Target(ResolvedCallTarget::ModuleExport {
                specifier,
                export_path,
                ..
            }) if !specifier.is_empty() => FlowValue::Target(ResolvedCallTarget::ModuleExport {
                specifier,
                export_path: if export_path.is_empty() {
                    member.to_string()
                } else {
                    format!("{export_path}.{member}")
                },
                repository_target: None,
                callable_id: None,
            }),
            _ => FlowValue::Unknown,
        }
    }
}
