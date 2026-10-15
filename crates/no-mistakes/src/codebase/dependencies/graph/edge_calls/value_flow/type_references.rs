use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn annotation_reference_view<'q>(
        &'q self,
        file: &'q std::path::Path,
        reference: &str,
        scope: AnnotationScope<'q>,
        guards: &mut TypeQueryGuards<'q>,
    ) -> Option<AnnotationView<'q>> {
        let facts = self.facts.get_ts_facts(file)?;
        let (root, namespace) = reference
            .split_once('.')
            .map_or((reference, None), |(root, member)| (root, Some(member)));
        let local = self.annotation_binding_scope(file, scope.lexical, root)?;
        if local.is_some_and(|scope| scope != 0) {
            return None;
        }
        if let Some(imported) = facts
            .imported_bindings
            .iter()
            .find(|binding| binding.local == root)
            .filter(|_| local.is_none())
        {
            let export = super::super::provenance::type_export(imported, namespace)?;
            return self.imported_annotation_view(file, &imported.specifier, &export, guards);
        }
        let identity = namespace
            .is_none()
            .then(|| {
                let binding = facts
                    .value_flow
                    .type_exports
                    .iter()
                    .find(|binding| binding.local == root && binding.specifier.is_none())?;
                Some((scope.module?.to_owned(), binding.exported.clone()))
            })
            .flatten();
        if let Some(fields) = facts
            .value_flow
            .type_sources
            .as_ref()
            .and_then(|sources| sources.declarations.get(reference))
        {
            return Some(AnnotationView::Object {
                file,
                fields,
                scope: AnnotationScope {
                    lexical: 0,
                    ..scope
                },
                identity,
            });
        }
        identity.map(|(module, export)| AnnotationView::Origin(module, export))
    }
    fn imported_annotation_view<'q>(
        &'q self,
        file: &'q std::path::Path,
        module: &'q str,
        export: &str,
        guards: &mut TypeQueryGuards<'q>,
    ) -> Option<AnnotationView<'q>> {
        // Terminal imports retain their existing identity even without a local
        // shape. Only selecting a field requires a prepared object declaration.
        let origin = self.type_origin(file, module.to_owned(), export.to_owned());
        let terminal = || Some(AnnotationView::Origin(origin.0.clone(), origin.1.clone()));
        let Some(target) = self
            .resolver
            .resolve(module, file)
            .and_then(|target| self.inputs.graph_files.visible_path(&target))
        else {
            return terminal();
        };
        let Some(facts) = self.facts.get_ts_facts(target) else {
            return terminal();
        };
        let Some(binding) = facts
            .value_flow
            .type_exports
            .iter()
            .find(|binding| binding.exported == export)
        else {
            return terminal();
        };
        let key = (target, export.to_owned());
        if !guards.exports.insert(key.clone()) {
            // A cyclic shape cannot supply fields, but bare imported identity
            // still follows the existing terminal provenance contract.
            return terminal();
        }
        let result = if let Some(specifier) = &binding.specifier {
            self.imported_annotation_view(target, specifier, &binding.local, guards)
        } else if let Some(fields) = facts
            .value_flow
            .type_sources
            .as_ref()
            .and_then(|sources| sources.declarations.get(&binding.local))
        {
            Some(AnnotationView::Object {
                file: target,
                fields,
                scope: AnnotationScope {
                    lexical: 0,
                    module: Some(module),
                },
                identity: Some(origin),
            })
        } else {
            terminal()
        };
        guards.exports.remove(&key);
        result
    }
}
