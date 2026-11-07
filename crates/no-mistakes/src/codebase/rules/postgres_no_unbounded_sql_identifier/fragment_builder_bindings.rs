use super::*;
use crate::codebase::dependencies::extract::{CallableId, ValueBinding};

impl Proof<'_, '_> {
    pub(in super::super) fn binding(
        &mut self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        default_only: bool,
    ) -> bool {
        if self.file(file).is_none() {
            return false;
        }
        let Some(binding) = self.bindings.value(file, scope, name, at) else {
            return scope == 0 && self.imported(file, name);
        };
        self.local(file, binding, default_only, false)
    }

    pub(in super::super) fn namespace(&mut self, file: &Path, local: &str, exported: &str) -> bool {
        if self.file(file).is_none() {
            return false;
        }
        let Some(ts) = self.facts.get_ts_facts(file) else {
            return false;
        };
        if ts
            .value_flow
            .bindings
            .iter()
            .any(|binding| binding.name == local)
            || ts
                .value_flow
                .parameters
                .iter()
                .any(|parameter| parameter.name == local)
        {
            return false;
        }
        self.imported(file, &format!("{local}.{exported}"))
    }

    fn imported(&mut self, file: &Path, selector: &str) -> bool {
        let Some((producer, name)) = self.graph.imported_value(file, selector) else {
            return false;
        };
        self.binding(&producer, 0, &name, u32::MAX, true)
    }

    fn local(
        &mut self,
        file: &Path,
        binding: &ValueBinding,
        default_only: bool,
        returned: bool,
    ) -> bool {
        let key = Key::Binding(
            file.to_path_buf(),
            binding.scope,
            binding.name.clone(),
            binding.declared_at,
            default_only,
            returned,
        );
        self.memoized(key, |proof| {
            let Some(flow) = proof.file(file).map(|data| data.flow) else {
                return false;
            };
            if !binding.immutable
                || binding.invalidated_at.is_some()
                || flow
                    .identifier_writes
                    .iter()
                    .any(|write| write.scope == binding.scope && write.name == binding.name)
            {
                return false;
            }
            let exported = binding.scope == 0
                && proof.facts.get_ts_facts(file).is_some_and(|ts| {
                    ts.exported_bindings
                        .iter()
                        .any(|export| export.specifier.is_none() && export.local == binding.name)
                });
            if !proof.expression(file, &binding.value, default_only)
                || !proof.lifecycle(file, binding)
            {
                return false;
            }
            if exported {
                imports::safe(proof, file, binding)
            } else if returned {
                imports::owned_reads(proof, file, binding)
            } else {
                !flow
                    .registry_mutations
                    .iter()
                    .any(|write| write.scope == binding.scope && write.name == binding.name)
            }
        })
    }

    pub(super) fn returned(
        &mut self,
        file: &Path,
        owner: CallableId,
        value: &ValueExpression,
    ) -> bool {
        if let ValueExpression::Named {
            name,
            scope: Some(scope),
            offset,
            ..
        } = value
        {
            if let Some(binding) = self
                .bindings
                .value(file, *scope, name, *offset)
                .filter(|binding| binding.owner == Some(owner))
            {
                return self.local(file, binding, true, true);
            }
        }
        self.expression(file, value, true)
    }
}
