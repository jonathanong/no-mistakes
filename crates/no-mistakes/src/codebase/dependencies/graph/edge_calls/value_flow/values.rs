use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn value(
        &mut self,
        path: &std::path::Path,
        expression: &ValueExpression,
    ) -> FlowValue {
        self.value_projected(path, expression, None)
    }
    pub(super) fn value_projected(
        &mut self,
        path: &std::path::Path,
        expression: &ValueExpression,
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> FlowValue {
        let outer = self.default_origins.take();
        let value = self.expression_value(path, expression, projection);
        if value.is_undefined() {
            self.default_origins = None;
        }
        default_origins::merge(&mut self.default_origins, outer, self.cache);
        value
    }
    fn expression_value(
        &mut self,
        path: &std::path::Path,
        expression: &ValueExpression,
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> FlowValue {
        match expression {
            ValueExpression::Named {
                name,
                scope,
                offset,
                context,
            } => self.named(path, name, *scope, *offset, *context, projection),
            ValueExpression::Callable(id) => self.callable(path, *id),
            ValueExpression::Super(owner) => self.super_constructor(path, *owner),
            ValueExpression::Member(base, member) => {
                if let Some(value) = self.replacement_member(path, base, member) {
                    return value;
                }
                let read_location = Self::member_read_location(base);
                let hint = member_projection::MemberProjection {
                    member,
                    outer: projection,
                };
                let base = self.value_projected(path, base, Some(&hint));
                self.member_at(base, member, path, read_location)
            }
            ValueExpression::Object(fields, null_prototype) => FlowValue::Object {
                null_prototype: *null_prototype,
                file: path.to_path_buf(),
                fields: std::sync::Arc::clone(fields),
            },
            ValueExpression::Array(fields) => FlowValue::Array {
                file: path.to_path_buf(),
                fields: std::sync::Arc::clone(fields),
            },
            ValueExpression::Undefined => FlowValue::Undefined,
            ValueExpression::Null => FlowValue::Null,
            ValueExpression::Nullish(left, right) => {
                let left = self.value(path, left);
                if matches!(left.executor(), FlowValue::Undefined | FlowValue::Null) {
                    return self.value(path, right);
                }
                if !matches!(left.executor(), FlowValue::Unknown | FlowValue::Choice(_)) {
                    return left;
                }
                let right = self.value(path, right);
                FlowValue::Choice(vec![left, right])
            }
            ValueExpression::Default(value, fallback) => {
                let value = self.value(path, value);
                if value.is_undefined() {
                    self.value(path, fallback)
                } else {
                    value
                }
            }
            ValueExpression::Awaited(value) => self.value(path, value).awaited(),
            ValueExpression::Call(offset, end) => self.returned(path, *offset, *end),
            // A conditional element used to be Unknown. Keep that flow value.
            ValueExpression::Either(_, _) | ValueExpression::Unknown => FlowValue::Unknown,
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
}

#[path = "value_members.rs"]
mod members;
