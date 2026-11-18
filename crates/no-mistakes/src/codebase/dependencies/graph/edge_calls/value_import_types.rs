fn project_identifier_type_imports(
    projection: &ValueImportProjection<'_, '_>,
    file: &Path,
    ts: &crate::codebase::ts_source::facts::TsFileFacts,
    edges: &mut Vec<Edge>,
) {
    use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueType};

    if ts.imported_bindings.is_empty() {
        return;
    }
    let flow = &ts.value_flow;
    let mut imports = fx_map();
    for binding in &ts.imported_bindings {
        imports
            .entry(binding.local.as_str())
            .and_modify(|existing| *existing = None)
            .or_insert(Some(binding));
    }
    let mut targets = fx_map();
    let mut demands = fx_map();
    let shapes = flow
        .type_shapes
        .values()
        .chain(
            flow.parameters
                .iter()
                .filter_map(|param| param.type_reference.as_ref()),
        )
        .chain(flow.widened_string_assertions.values().flatten());
    for shape in shapes {
        let (name, role) = match shape {
            ValueType::Element(name) => (name.as_str(), 1u8),
            ValueType::Reference(name) => (name.as_str(), 2u8),
            _ => continue,
        };
        *demands.entry(name).or_insert(0) |= role;
    }
    for (name, role) in demands {
        if role == 3 {
            continue;
        }
        let Some(Some(binding)) = imports.get(name) else {
            continue;
        };
        if binding.kind == ImportedBindingKind::Namespace {
            continue;
        }
        let target = targets
            .entry(binding.specifier.as_str())
            .or_insert_with(|| projection.type_module_target(file, &binding.specifier));
        let Some(target) = target.as_deref() else {
            continue;
        };
        let (producer, local) = if role == 1 {
            let ExportedCallableResolution::Value(producer, local) =
                projection.resolve(target, &binding.imported)
            else {
                continue;
            };
            if !projection.live_value(&producer, &local) {
                continue;
            }
            (producer, local)
        } else {
            let Some((producer, local)) =
                projection.type_export(target, &binding.imported, &mut fx_set())
            else {
                continue;
            };
            (producer, local)
        };
        edges.push((
            NodeId::symbol_in(&projection.inputs.interner, file, name),
            NodeId::symbol_in(&projection.inputs.interner, &producer, local),
            EdgeKind::SymbolTypeImport,
        ));
    }
}

impl ValueImportProjection<'_, '_> {
    fn type_module_target(&self, file: &Path, specifier: &str) -> Option<PathBuf> {
        self.resolver.resolve(specifier, file).and_then(|target| {
            self.inputs.graph_files.visible_path(&target).map(Path::to_path_buf)
        })
    }

    fn type_export(
        &self,
        file: &Path,
        public: &str,
        seen: &mut FxHashSet<(PathBuf, String)>,
    ) -> Option<(PathBuf, String)> {
        use crate::codebase::dependencies::extract::ImportedBindingKind;

        if !seen.insert((file.to_path_buf(), public.to_owned())) {
            return None;
        }
        let facts = self
            .facts
            .get_ts_facts(file)
            .filter(|facts| value_import_facts_complete(facts))?;
        let flow = &facts.value_flow;
        let mut exports = flow
            .type_exports
            .iter()
            .filter(|export| export.exported == public);
        let export = exports.next()?;
        if exports.next().is_some() {
            return None;
        }
        if let Some(specifier) = &export.specifier {
            let target = self.type_module_target(file, specifier)?;
            return self.type_export(&target, &export.local, seen);
        }
        if flow.type_shapes.contains_key(&export.local) {
            return Some((file.to_path_buf(), export.local.clone()));
        }
        let mut imports = facts
            .imported_bindings
            .iter()
            .filter(|binding| binding.local == export.local);
        let binding = imports.next()?;
        if imports.next().is_some() || binding.kind == ImportedBindingKind::Namespace {
            return None;
        }
        let target = self.type_module_target(file, &binding.specifier)?;
        self.type_export(&target, &binding.imported, seen)
    }
}
