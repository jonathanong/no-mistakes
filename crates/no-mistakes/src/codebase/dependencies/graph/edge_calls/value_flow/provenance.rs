use super::*;
use crate::codebase::dependencies::extract::ImportedBindingKind;

impl ValueFlowResolution<'_, '_> {
    #[cfg(all(test, feature = "test-instrumentation"))]
    pub(super) fn test_computation_scope() -> impl Drop {
        tests::lifetime::scope()
    }
    #[cfg(all(test, feature = "test-instrumentation"))]
    pub(super) fn test_old_annotation_lifetime() -> bool {
        tests::lifetime::old_mode()
    }
    #[cfg(all(test, feature = "test-instrumentation"))]
    pub(super) fn test_retiring_computation_buffers(&self) {
        tests::lifetime::retiring(
            self.visiting.capacity()
                + self.calls_visiting.capacity()
                + self.initializing.capacity(),
        );
    }

    pub(super) fn typed_parameter(
        &self,
        path: &std::path::Path,
        parameter: &ValueParameter,
        _index: &CallableFileIndex,
        executor: FlowValue,
    ) -> FlowValue {
        let Some(reference) = &parameter.type_reference else {
            return executor;
        };
        #[cfg(all(test, feature = "test-instrumentation"))]
        tests::record_annotation();
        let annotation = self.annotation_value(path, reference, None, &mut fx_set());
        Self::annotate(executor, annotation)
    }

    // Type re-exports use the same prepared fact catalog as calls. Walking one
    // explicit binding chain does not widen runtime reachability or parse again.
    pub(super) fn type_origin(
        &self,
        path: &std::path::Path,
        mut module: String,
        mut export: String,
    ) -> (String, String) {
        let mut path = path.to_path_buf();
        let mut visited = fx_set();
        while let Some(target) = self.resolver.resolve(&module, &path).and_then(|target| {
            self.inputs
                .graph_files
                .visible_path(&target)
                .map(std::path::Path::to_path_buf)
        }) {
            if !visited.insert((target.clone(), export.clone())) {
                break;
            }
            let Some(facts) = self.facts.get_ts_facts(&target) else {
                break;
            };
            // Barrels bind the exported root; the remaining namespace path stays intact.
            let (root, member) = export
                .split_once('.')
                .map_or((export.as_str(), None), |(root, member)| {
                    (root, Some(member))
                });
            let Some(binding) = facts
                .value_flow
                .type_exports
                .iter()
                .find(|binding| binding.exported == root)
            else {
                break;
            };
            if let Some(specifier) = &binding.specifier {
                module = specifier.clone();
                export = member.map_or_else(
                    || binding.local.clone(),
                    |member| format!("{}.{member}", binding.local),
                );
            } else {
                let Some(imported) = facts
                    .imported_bindings
                    .iter()
                    .find(|imported| imported.local == binding.local)
                else {
                    break;
                };
                let Some(next) = type_export(imported, member) else {
                    break;
                };
                module = imported.specifier.clone();
                export = next;
            }
            path = target;
        }
        (module, export)
    }

    pub(super) fn project_provenance(
        &mut self,
        executor: FlowValue,
        _typed: Vec<(String, String)>,
        members: Vec<(String, String, String)>,
        factories: Vec<(String, String)>,
        member: &str,
    ) -> FlowValue {
        // A known promise or non-receiver cannot supply a configured member.
        if matches!(
            executor.executor(),
            FlowValue::Promise(_) | FlowValue::Undefined | FlowValue::Uncallable
        ) {
            return self.member_value(executor, member);
        }
        let executor = self.member_value(executor, member);
        if members.is_empty() && factories.is_empty() {
            return executor;
        }
        FlowValue::Provenance {
            executor: Box::new(executor),
            typed: Vec::new(),
            members: members
                .into_iter()
                .map(|(module, export, prefix)| (module, export, format!("{prefix}.{member}")))
                .chain(
                    factories
                        .into_iter()
                        .map(|(module, export)| (module, export, member.to_owned())),
                )
                .collect(),
            receiver_factories: Vec::new(),
        }
    }
}

pub(super) fn type_export(
    binding: &crate::codebase::dependencies::extract::ImportedBinding,
    member: Option<&str>,
) -> Option<String> {
    match binding.kind {
        ImportedBindingKind::Namespace => member.map(str::to_owned),
        ImportedBindingKind::Named | ImportedBindingKind::Default => {
            let export = if binding.kind == ImportedBindingKind::Default {
                "default"
            } else {
                &binding.imported
            };
            Some(member.map_or_else(|| export.to_owned(), |member| format!("{export}.{member}")))
        }
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "parameter_annotation_tests.rs"]
mod tests;
