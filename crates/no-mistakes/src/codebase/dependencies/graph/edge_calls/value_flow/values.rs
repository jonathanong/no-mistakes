use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn value(
        &mut self,
        path: &std::path::Path,
        expression: &ValueExpression,
    ) -> FlowValue {
        let outer = self.default_origins.take();
        let value = self.expression_value(path, expression);
        if value == FlowValue::Undefined {
            self.default_origins = None;
        }
        default_origins::merge(&mut self.default_origins, outer);
        value
    }
    fn expression_value(
        &mut self,
        path: &std::path::Path,
        expression: &ValueExpression,
    ) -> FlowValue {
        match expression {
            ValueExpression::Named {
                name,
                scope,
                offset,
                context,
            } => self.named(path, name, *scope, *offset, *context),
            ValueExpression::Callable(id) => self.callable(path, *id),
            ValueExpression::Super(owner) => self.super_constructor(path, *owner),
            ValueExpression::Member(base, member) => {
                let base = self.value(path, base);
                self.member(base, member)
            }
            ValueExpression::Object(fields) => FlowValue::Object {
                file: path.to_path_buf(),
                fields: std::sync::Arc::clone(fields),
            },
            ValueExpression::Array(fields) => FlowValue::Array {
                file: path.to_path_buf(),
                fields: std::sync::Arc::clone(fields),
            },
            ValueExpression::Undefined => FlowValue::Undefined,
            ValueExpression::Default(value, fallback) => {
                let value = self.value(path, value);
                if value == FlowValue::Undefined {
                    self.value(path, fallback)
                } else {
                    value
                }
            }
            ValueExpression::Awaited(value) => {
                let mut value = self.value(path, value);
                while let FlowValue::Promise(inner) = value {
                    value = *inner;
                }
                value
            }
            ValueExpression::Call(offset, end) => self.returned(path, *offset, *end),
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
    pub(super) fn member(&mut self, base: FlowValue, member: &str) -> FlowValue {
        self.project_default_origins(member);
        self.member_value(base, member)
    }
    fn member_value(&mut self, base: FlowValue, member: &str) -> FlowValue {
        match base {
            FlowValue::Returned {
                receiver: Some(receiver),
                ..
            } => self.member_value(FlowValue::Target(receiver), member),
            FlowValue::Array { file, fields } if member.parse::<usize>().is_ok() => fields
                .get(member)
                .map_or(FlowValue::Undefined, |value| self.value(&file, value)),
            FlowValue::Object { file, fields } => fields
                .get(member)
                .map_or(FlowValue::Undefined, |value| self.value(&file, value)),
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
