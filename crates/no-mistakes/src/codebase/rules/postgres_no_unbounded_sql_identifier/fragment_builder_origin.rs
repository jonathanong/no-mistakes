use super::*;
use crate::codebase::dependencies::extract::{
    CallableId, CatalogReadKind, InvocationKind, ValueParameter,
};
use crate::codebase::dependencies::graph::NodeId;

pub(super) type Callers = FxHashMap<PathBuf, FxHashMap<CallableId, Vec<usize>>>;

impl Proof<'_, '_> {
    pub(super) fn origin(&mut self, file: &Path, value: &ValueExpression) -> bool {
        match value {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => {
                let scope = scope.unwrap_or(0);
                self.memoized(
                    Key::Origin(file.to_path_buf(), scope, name.clone(), *offset),
                    |proof| {
                        let Some(flow) = proof.file(file).map(|data| data.flow) else {
                            return false;
                        };
                        if flow
                            .identifier_writes
                            .iter()
                            .any(|write| write.scope == scope && write.name == *name)
                            || !proof.origin_reads(file, scope, name)
                        {
                            return false;
                        }
                        if let Some(binding) = proof.bindings.value(file, scope, name, *offset) {
                            return binding.immutable
                                && binding.invalidated_at.is_none()
                                && proof.origin(file, &binding.value);
                        }
                        let Some(parameter) = flow
                            .parameters
                            .iter()
                            .find(|parameter| parameter.scope == scope && parameter.name == *name)
                        else {
                            return false;
                        };
                        proof.parameter_origin(file, parameter)
                    },
                )
            }
            ValueExpression::Call(start, end) => {
                let Some(call) = self
                    .file(file)
                    .and_then(|data| data.calls.get(&(*start, *end)))
                    .copied()
                else {
                    return false;
                };
                if self.files[file]
                    .flow
                    .tagged_templates
                    .contains_key(&(*start, *end))
                {
                    return crate::codebase::rules::postgres_sql_cross_file::is_text_tag(
                        self.bindings,
                        file,
                        call,
                        &[],
                    );
                }
                !call.bound
                    && call.spread_at.is_none()
                    && call.arguments.len() == 1
                    && matches!(&call.callee, ValueExpression::Member(base, method)
                        if method == "append" && self.origin(file, base))
            }
            _ => false,
        }
    }

    pub(super) fn origin_reads(&self, file: &Path, scope: usize, name: &str) -> bool {
        let flow = self.files[file].flow;
        flow.catalogs.reads.iter().filter(|(_, read)| read.scope == scope && read.name == name)
            .all(|(at, read)| {
                imports::reads::readonly(read, &read.path)
                    || read.path.as_slice() == ["append"] && read.kind == CatalogReadKind::Invalid
                    || read.path.is_empty() && self.facts.embedded_sql(file, &self.opts.embedded)
                        .is_ok_and(|sql| sql.call_spans.iter().any(|span| {
                            self.files[file].calls.get(span).is_some_and(|call|
                                matches!(call.arguments.first(), Some(ValueExpression::Named { name: actual, scope: found, offset, .. })
                                    if actual == name && found.unwrap_or(0) == scope && offset == at))
                        }))
            })
    }

    fn parameter_origin(&mut self, file: &Path, parameter: &ValueParameter) -> bool {
        if parameter.invalidated_at.is_some() || !parameter.projection.is_empty() {
            return false;
        }
        if self.callers.is_none() {
            let mut callers: Callers = FxHashMap::default();
            for (index, site) in self.graph.resolved_call_sites().iter().enumerate() {
                for target in site.targets() {
                    if let Some(NodeId::Symbol {
                        file,
                        callable_id: Some(owner),
                        ..
                    }) = target.node
                    {
                        callers
                            .entry(file.to_path_buf())
                            .or_default()
                            .entry(*owner)
                            .or_default()
                            .push(index);
                    }
                }
            }
            self.callers = Some(callers);
        }
        let sites = self
            .callers
            .as_ref()
            .and_then(|callers| callers.get(file))
            .and_then(|callers| callers.get(&parameter.owner))
            .cloned()
            .unwrap_or_default();
        !sites.is_empty()
            && sites.iter().all(|index| {
                let site = &self.graph.resolved_call_sites()[*index];
                if site.invocation != InvocationKind::Call
                    || site.has_unknown_target()
                    || site.targets().count() != 1
                    || self.graph.failed_call_sources(site).next().is_some()
                {
                    return false;
                }
                let Some(call) = self
                    .file(&site.file)
                    .and_then(|data| data.calls_at.get(&(site.offset, site.caller_id)))
                    .copied()
                else {
                    return false;
                };
                !call.bound
                    && call.spread_at.is_none()
                    && !matches!(&call.callee, ValueExpression::Member(_, method)
                        if matches!(method.as_str(), "call" | "apply" | "bind"))
                    && call
                        .arguments
                        .get(parameter.index)
                        .is_some_and(|argument| self.origin(&site.file, argument))
            })
    }
}
