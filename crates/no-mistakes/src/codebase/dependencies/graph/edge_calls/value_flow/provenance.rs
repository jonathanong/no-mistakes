use super::*;
use crate::codebase::dependencies::extract::ImportedBindingKind;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn typed_parameter(
        &self,
        path: &std::path::Path,
        parameter: &ValueParameter,
        index: &CallableFileIndex,
        executor: FlowValue,
    ) -> FlowValue {
        let Some(reference) = &parameter.type_reference else {
            return executor;
        };
        let (local, member) = reference
            .split_once('.')
            .map_or((reference.as_str(), None), |(a, b)| (a, Some(b)));
        let Some(binding) = index.type_imported.get(local) else {
            return executor;
        };
        let Some(export) = type_export(binding, member) else {
            return executor;
        };
        let origin = self
            .cache
            .type_origin((path.to_path_buf(), reference.clone()), || {
                self.type_origin(path, binding.specifier.clone(), export)
            });
        FlowValue::Provenance {
            executor: Box::new(executor),
            typed: vec![origin],
            members: Vec::new(),
            receiver_factories: Vec::new(),
        }
    }

    // Type re-exports use the same prepared fact catalog as calls. Walking one
    // explicit binding chain does not widen runtime reachability or parse again.
    fn type_origin(
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

fn type_export(
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
