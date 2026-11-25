use super::*;

impl ExportLookup<'_> {
    pub(super) fn resolve_wrappers(
        &self,
        from: &Path,
        calls: &[WrapperCall],
    ) -> Option<Vec<(ExportKey, usize)>> {
        calls
            .iter()
            .map(|call| Some((self.local(from, &call.callee)?, call.arg_index)))
            .collect()
    }

    pub(super) fn wrapper_export(
        &self,
        facts: Option<&CheckFileFacts>,
        key: &ExportKey,
    ) -> WrapperProof {
        let Some(facts) = facts.filter(|file| file.parse_error.is_none() && file.parsed) else {
            return WrapperProof::Unknown;
        };
        let ts = &facts.ts;
        if let Some(binding) = ts
            .exported_bindings
            .iter()
            .find(|binding| binding.exported == key.name.as_ref())
        {
            if let Some(specifier) = &binding.specifier {
                return self
                    .local(&key.file, &ImportRef::new(specifier, &binding.local))
                    .map_or(WrapperProof::Unknown, WrapperProof::Alias);
            }
            if let Some(summary) = facts
                .fixture_initializers
                .as_ref()
                .and_then(|fixtures| fixtures.wrappers.get(&binding.local))
            {
                let Some(deps) = self.resolve_wrappers(&key.file, &summary.wrappers) else {
                    return WrapperProof::Unknown;
                };
                return WrapperProof::Direct {
                    param_index: summary.param_index,
                    deps,
                };
            }
            if let Some(import) = ts.imported_bindings.iter().find(|import| {
                !import.is_type_only
                    && import.kind != ImportedBindingKind::Namespace
                    && import.local == binding.local
            }) {
                return self
                    .local(
                        &key.file,
                        &ImportRef::new(&import.specifier, &import.imported),
                    )
                    .map_or(WrapperProof::Unknown, WrapperProof::Alias);
            }
            return WrapperProof::Unknown;
        }
        if key.name.as_ref() == "default" {
            return WrapperProof::Unknown;
        }
        let targets: Option<Vec<_>> = ts
            .star_reexport_specifiers
            .iter()
            .map(|specifier| self.local(&key.file, &ImportRef::new(specifier, &key.name)))
            .collect();
        targets.map_or(WrapperProof::Unknown, WrapperProof::Stars)
    }
}
