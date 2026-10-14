use super::*;

/// Receiver identity is resolved once for a hot file, before property phases
/// are indexed. Different source names for one export share the same identity.
#[derive(Clone, PartialEq, Eq, Hash)]
enum ReceiverIdentity {
    External(String, String),
    Callable(std::path::PathBuf, CallableId),
}
#[derive(Default)]
pub(super) struct ReceiverIdentities {
    identities: FxHashMap<ReceiverIdentity, usize>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) normalizations: usize,
    #[cfg(feature = "test-instrumentation")]
    pub(super) imported_ids: FxHashSet<usize>,
}
impl ReceiverIdentities {
    pub(super) fn into_mutated_external(
        self,
        writers: &super::writers::ForeignWriters<'_>,
    ) -> FxHashMap<String, FxHashMap<String, (FxHashSet<String>, bool)>> {
        let mut mutated: FxHashMap<String, FxHashMap<String, (FxHashSet<String>, bool)>> = fx_map();
        for (identity, receiver) in self.identities {
            if let ReceiverIdentity::External(module, export) = identity {
                if let Some(summary) = writers.summary(receiver) {
                    mutated.entry(module).or_default().insert(export, summary);
                }
            }
        }
        mutated
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn local_ids(&self) -> usize {
        self.identities.len() - self.imported_ids.len()
    }
    pub(super) fn file(
        &mut self,
        inputs: &GraphEdgeBuildInputs<'_>,
        facts: &dyn TsFactLookup,
        resolver: &dyn ImportResolution,
        indexes: &CallableResolutionIndexes,
        path: &std::path::Path,
        index: &CallableFileIndex,
    ) -> FxHashMap<String, usize> {
        let mut receivers = fx_map();
        for binding in index.imported.values() {
            if binding.is_type_only {
                continue;
            }
            #[cfg(feature = "test-instrumentation")]
            {
                self.normalizations += 1;
            }
            let Some(identity) = receiver_identity(inputs, facts, resolver, indexes, path, binding)
            else {
                continue;
            };
            let next = self.identities.len();
            let id = *self.identities.entry(identity).or_insert(next);
            receivers.insert(binding.local.clone(), id);
            #[cfg(feature = "test-instrumentation")]
            {
                self.imported_ids.insert(id);
            }
        }
        if let Some(bindings) = index.callable_bindings.get(&0) {
            let mut add = |name: &String, id: &CallableId| {
                let next = self.identities.len();
                let receiver = *self
                    .identities
                    .entry(ReceiverIdentity::Callable(path.to_path_buf(), *id))
                    .or_insert(next);
                receivers.entry(name.clone()).or_insert(receiver);
            };
            match bindings {
                ScopedNames::One(name, id) => add(name, id),
                ScopedNames::Many(bindings) => {
                    for (name, id) in bindings {
                        add(name, id);
                    }
                }
            }
        }
        receivers
    }
}
fn receiver_identity(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    path: &std::path::Path,
    binding: &crate::codebase::dependencies::extract::ImportedBinding,
) -> Option<ReceiverIdentity> {
    if binding.kind == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace {
        return None;
    }
    let Some(source) = resolver.resolve(&binding.specifier, path) else {
        return Some(ReceiverIdentity::External(
            binding.specifier.clone(),
            binding.imported.clone(),
        ));
    };
    match resolve_exported_callable(
        inputs,
        facts,
        resolver,
        &source,
        &binding.imported,
        indexes,
        &mut Vec::new(),
    ) {
        ExportedCallableResolution::ExternalModuleExport(module, export) => {
            Some(ReceiverIdentity::External(module, export))
        }
        ExportedCallableResolution::Callable(file, _, Some(id)) => {
            Some(ReceiverIdentity::Callable(file, id))
        }
        _ => None,
    }
}
