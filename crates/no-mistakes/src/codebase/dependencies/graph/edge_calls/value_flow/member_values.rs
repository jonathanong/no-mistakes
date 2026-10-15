use super::*;

impl ValueFlowResolution<'_, '_> {
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
            FlowValue::Annotated {
                executor,
                file,
                source,
            } => FlowValue::Annotated {
                executor: Box::new(self.member_value(*executor, member)),
                file,
                source: std::sync::Arc::new(
                    crate::codebase::dependencies::extract::ValueTypeSource::Indexed(
                        Box::new((*source).clone()),
                        member.to_owned(),
                    ),
                ),
            },
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
