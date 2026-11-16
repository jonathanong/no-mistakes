use super::*;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::NodeId;

impl Proof<'_, '_> {
    pub(in super::super) fn binds_namespace(
        &mut self,
        file: &Path,
        local: &str,
        exported: &str,
    ) -> bool {
        self.graph
            .imported_value(file, &format!("{local}.{exported}"))
            .is_none_or(|(producer, name)| self.binds_at(&producer, 0, &name, u32::MAX))
    }
    pub(in super::super) fn binds_at(
        &mut self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
    ) -> bool {
        self.binds_binding(
            file,
            scope,
            name,
            at,
            &mut FxHashSet::default(),
            &mut FxHashSet::default(),
        )
    }

    fn binds_binding(
        &mut self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        seen: &mut FxHashSet<(PathBuf, usize, String, u32)>,
        calls: &mut FxHashSet<(PathBuf, u32, u32)>,
    ) -> bool {
        let Some(binding) = self.bindings.value(file, scope, name, at) else {
            if !seen.insert((file.to_path_buf(), scope, name.into(), u32::MAX)) {
                return true;
            }
            return scope != 0
                || self
                    .graph
                    .imported_value(file, name)
                    .is_none_or(|(producer, imported)| {
                        self.binds_binding(&producer, 0, &imported, u32::MAX, seen, calls)
                    });
        };
        if !seen.insert((file.to_path_buf(), scope, name.into(), binding.declared_at)) {
            return true;
        }
        let value = binding.value.clone();
        if self.binds_expression(file, &value, seen, calls) {
            return true;
        }
        let Some(flow) = self.file(file).map(|data| data.flow) else {
            return true;
        };
        let appended: Vec<_> = flow
            .calls
            .iter()
            .filter(|call| {
                matches!(&call.callee, ValueExpression::Member(owner, method)
                if method == "append" && matches!(owner.as_ref(), ValueExpression::Named {
                    name: other, scope: other_scope, ..
                } if other == name && other_scope.unwrap_or(0) == scope))
            })
            .filter_map(|call| call.arguments.first().cloned())
            .collect();
        appended
            .iter()
            .any(|argument| self.binds_expression(file, argument, seen, calls))
    }

    fn binds_expression(
        &mut self,
        file: &Path,
        value: &ValueExpression,
        seen: &mut FxHashSet<(PathBuf, usize, String, u32)>,
        calls: &mut FxHashSet<(PathBuf, u32, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.binds_binding(file, scope.unwrap_or(0), name, *offset, seen, calls),
            ValueExpression::Call(start, end) => {
                if !calls.insert((file.to_path_buf(), *start, *end)) {
                    return true;
                }
                let Some(call) = self
                    .file(file)
                    .and_then(|data| data.calls.get(&(*start, *end)))
                    .copied()
                else {
                    return true;
                };
                if self.files[file]
                    .flow
                    .tagged_templates
                    .contains_key(&(*start, *end))
                {
                    return call.arguments.len() > 1;
                }
                if let ValueExpression::Member(base, method) = &call.callee {
                    return method != "append"
                        || self.binds_expression(file, base, seen, calls)
                        || call
                            .arguments
                            .iter()
                            .any(|arg| self.binds_expression(file, arg, seen, calls));
                }
                let targets = self
                    .graph
                    .call_sites_in_file(file)
                    .iter()
                    .filter(|site| {
                        site.offset == *start
                            && site.caller_id == call.context
                            && site.invocation == InvocationKind::Call
                            && !site.has_unknown_target()
                    })
                    .flat_map(|site| site.targets())
                    .filter_map(|target| match target.node {
                        Some(NodeId::Symbol {
                            file,
                            callable_id: Some(owner),
                            ..
                        }) => Some((file.to_path_buf(), *owner)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                targets.is_empty()
                    || targets.iter().any(|(producer, owner)| {
                        let returns = self
                            .file(producer)
                            .map(|data| {
                                data.flow
                                    .returns
                                    .iter()
                                    .filter(|(id, _)| id == owner)
                                    .map(|(_, value)| value.clone())
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        returns.is_empty()
                            || returns
                                .iter()
                                .any(|value| self.binds_expression(producer, value, seen, calls))
                    })
            }
            _ => true,
        }
    }
}
