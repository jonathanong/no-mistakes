use super::*;
use crate::codebase::dependencies::extract::{InvocationKind, ValueBinding};
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
        self.binds_binding(file, scope, name, at)
    }

    fn binds_binding(&mut self, file: &Path, scope: usize, name: &str, at: u32) -> bool {
        let binding = self.bindings.value(file, scope, name, at);
        // The builder proof has already rejected cycles and untracked calls.
        if let Some(binding) = binding {
            self.bound_binding(file, binding, at)
        } else {
            scope != 0
                || self
                    .graph
                    .imported_value(file, name)
                    .is_none_or(|(producer, imported)| {
                        self.binds_binding(&producer, 0, &imported, u32::MAX)
                    })
        }
    }

    fn bound_binding(&mut self, file: &Path, binding: &ValueBinding, at: u32) -> bool {
        if self.binds_expression(file, &binding.value) {
            return true;
        }
        let flow = self.files[file].flow;
        let appended: Vec<_> = flow
            .calls
            .iter()
            .filter(|call| call.offset < at)
            .filter(|call| {
                matches!(&call.callee, ValueExpression::Member(owner, method)
                if method == "append" && self.append_receiver(file, owner, binding))
            })
            .filter_map(|call| call.arguments.first().cloned())
            .collect();
        appended
            .iter()
            .any(|argument| self.binds_expression(file, argument))
    }

    fn binds_expression(&mut self, file: &Path, value: &ValueExpression) -> bool {
        match value {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.binds_binding(file, scope.unwrap_or(0), name, *offset),
            ValueExpression::Call(start, end) => self.binds_call(file, *start, *end),
            _ => true,
        }
    }

    fn binds_call(&mut self, file: &Path, start: u32, end: u32) -> bool {
        let call = self.files[file].calls[&(start, end)];
        if self.files[file]
            .flow
            .tagged_templates
            .contains_key(&(start, end))
        {
            return call.arguments.len() > 1;
        }
        if let ValueExpression::Member(base, method) = &call.callee {
            return method != "append"
                || self.binds_expression(file, base)
                || call
                    .arguments
                    .iter()
                    .any(|arg| self.binds_expression(file, arg));
        }
        let targets = self
            .graph
            .call_sites_in_file(file)
            .iter()
            .filter(|site| {
                site.offset == start
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
                        .any(|value| self.binds_expression(producer, value))
            })
    }
}
