enum ImportedSelector {
    Missing,
    Value,
    Namespace(std::path::PathBuf),
}

struct ValueImportProjection<'a, 'b> {
    inputs: &'a GraphEdgeBuildInputs<'b>,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    indexes: &'a CallableResolutionIndexes,
}

impl ValueImportProjection<'_, '_> {
    fn resolve(&self, file: &std::path::Path, export: &str) -> ExportedCallableResolution {
        if export.is_empty() {
            return if self
                .facts
                .get_ts_facts(file)
                .is_some_and(value_import_facts_complete)
            {
                ExportedCallableResolution::Namespace(file.to_path_buf())
            } else {
                ExportedCallableResolution::Unknown
            };
        }
        let mut trace = ExportResolutionTrace::default();
        let result = resolve_exported_callable(
            self.inputs,
            self.facts,
            self.resolver,
            file,
            export,
            self.indexes,
            &mut trace,
        );
        if !trace.failures.is_empty()
            || matches!(&result,
            ExportedCallableResolution::Namespace(target)
                if !self.facts.get_ts_facts(target).is_some_and(value_import_facts_complete))
        {
            ExportedCallableResolution::Unknown
        } else {
            result
        }
    }

    fn selector(
        &self,
        export: (&std::path::Path, &str),
        import: (&std::path::Path, &str),
        edges: &mut Vec<Edge>,
    ) -> ImportedSelector {
        let result = match self.resolve(export.0, export.1) {
            ExportedCallableResolution::Value(producer, local) => {
                let Some(source) = self.indexes.file(self.facts, &producer) else {
                    return ImportedSelector::Missing;
                };
                if !source.values.immutable_binding(0, &local)
                    || !self
                        .facts
                        .get_ts_facts(&producer)
                        .is_some_and(value_import_facts_complete)
                {
                    return ImportedSelector::Missing;
                }
                (
                    NodeId::symbol_in(&self.inputs.interner, &producer, local),
                    ImportedSelector::Value,
                )
            }
            ExportedCallableResolution::Namespace(namespace) => (
                NodeId::file_in(&self.inputs.interner, &namespace),
                ImportedSelector::Namespace(namespace),
            ),
            _ => return ImportedSelector::Missing,
        };
        edges.push((
            NodeId::symbol_in(&self.inputs.interner, import.0, import.1),
            result.0,
            EdgeKind::SymbolImport,
        ));
        result.1
    }
}
