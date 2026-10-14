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
                let whole_receiver = std::cell::Cell::new(true);
                let (read_offset, read_context) = Self::member_read_location(base);
                let hint = member_projection::MemberProjection {
                    member,
                    outer: projection,
                    transparent_alias: false,
                    whole_receiver: projection.map_or(&whole_receiver, |hint| hint.whole_receiver),
                };
                let base = self.value_projected(path, base, Some(&hint));
                self.member_at(base, member, path, read_offset, read_context)
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
        let value = self.member_value(base, member);
        let value = if self.receiver_origin_invalidated() {
            FlowValue::Unknown
        } else {
            value
        };
        self.snapshot_receiver_origins();
        value
    }
    pub(super) fn member_value(&mut self, base: FlowValue, member: &str) -> FlowValue {
        match base {
            FlowValue::Returned {
                factory,
                executor,
                receiver,
            } => {
                let value = match receiver {
                    Some(receiver) => self.member_value(FlowValue::Target(receiver), member),
                    None => self.member_value(*executor, member),
                };
                FlowValue::Provenance {
                    executor: Box::new(value),
                    typed: Vec::new(),
                    members: factory
                        .into_iter()
                        .map(|(module, export)| (module, export, member.to_owned()))
                        .collect(),
                    receiver_factories: Vec::new(),
                }
            }
            FlowValue::Provenance {
                executor,
                typed,
                members,
                receiver_factories,
            } => self.project_provenance(*executor, typed, members, receiver_factories, member),
            FlowValue::Array { file, fields } if member.parse::<usize>().is_ok() => {
                fields.get(member).map_or(FlowValue::Undefined, |value| {
                    self.receiver_field_value(&file, value)
                })
            }

            FlowValue::Object {
                file,
                fields,
                null_prototype,
            } => fields.get(member).map_or_else(
                || {
                    if !null_prototype && inherited_object_member(member) {
                        FlowValue::Unknown
                    } else {
                        FlowValue::Undefined
                    }
                },
                |value| self.receiver_field_value(&file, value),
            ),
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

fn inherited_object_member(member: &str) -> bool {
    matches!(
        member,
        "constructor"
            | "toString"
            | "toLocaleString"
            | "valueOf"
            | "hasOwnProperty"
            | "isPrototypeOf"
            | "propertyIsEnumerable"
            | "__proto__"
            | "__defineGetter__"
            | "__defineSetter__"
            | "__lookupGetter__"
            | "__lookupSetter__"
    )
}
