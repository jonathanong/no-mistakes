use super::*;
use crate::codebase::dependencies::extract::ValueTypeSource;

#[derive(Clone, Copy, Default)]
pub(super) struct AnnotationScope<'a> {
    pub(super) lexical: usize,
    pub(super) module: Option<&'a str>,
}
impl ValueFlowResolution<'_, '_> {
    pub(super) fn annotation_binding_scope(
        &self,
        file: &std::path::Path,
        mut scope: usize,
        name: &str,
    ) -> Option<Option<usize>> {
        let facts = self.facts.get_ts_facts(file)?;
        let index = self.indexes.file(self.facts, file)?;
        loop {
            if facts
                .value_flow
                .type_sources
                .as_ref()
                .and_then(|sources| sources.bindings.get(&scope))
                .is_some_and(|names| names.contains(name))
                || scoped_name(&index.class_bindings, scope, name).is_some()
            {
                return Some(Some(scope));
            }
            match index.lexical_scope_parents.get(&scope)? {
                Some(parent) => scope = *parent,
                None => return Some(None),
            }
        }
    }
    pub(super) fn global_non_nullable(&self, file: &std::path::Path, scope: usize) -> Option<bool> {
        let facts = self.facts.get_ts_facts(file)?;
        Some(
            self.annotation_binding_scope(file, scope, "NonNullable")?
                .is_none()
                && !facts
                    .imported_bindings
                    .iter()
                    .any(|binding| binding.local == "NonNullable")
                && !facts
                    .value_flow
                    .type_exports
                    .iter()
                    .any(|binding| binding.local == "NonNullable" && binding.specifier.is_none()),
        )
    }
    pub(super) fn contextual_annotation_origin(
        &self,
        file: &std::path::Path,
        source: &ValueTypeSource,
        index: usize,
    ) -> Option<(String, String)> {
        let ValueTypeSource::Scoped { scope, source } = source else {
            return None;
        };
        let ValueTypeSource::Function(parameters) = source.as_ref() else {
            return None;
        };
        self.annotation_origin_scoped(
            file,
            parameters.get(index)?.as_ref()?,
            AnnotationScope {
                lexical: *scope,
                module: None,
            },
        )
    }
}
