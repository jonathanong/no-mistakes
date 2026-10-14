use super::*;
use default_origins::{merge, DefaultOriginUse};

impl ValueFlowResolution<'_, '_> {
    pub(super) fn record_default_origin(
        &mut self,
        file: &std::path::Path,
        parameter: &ValueParameter,
        origin: usize,
    ) {
        if parameter
            .default_mutations
            .as_ref()
            .is_some_and(|mutations| mutations.contains_key(&origin))
        {
            let source = self.cache.origin_node(
                OriginKind::Source(DefaultOriginUse {
                    file: file.to_path_buf(),
                    scope: parameter.scope,
                    name: parameter.name.clone(),
                    origin,
                }),
                1,
            );
            merge(&mut self.default_origins, Some(source), self.cache);
        }
    }
    pub(super) fn source_origin_invalidated(
        &self,
        origin: &DefaultOriginUse,
        projection: &[&str],
        file: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
        alias_seed: usize,
    ) -> bool {
        self.indexes
            .file(self.facts, &origin.file)
            .is_some_and(|index| {
                let Some(parameter) =
                    scoped_name(&index.values.parameters, origin.scope, &origin.name)
                else {
                    return false;
                };
                // A caller seed may refine a whole imported default receiver,
                // never a copied member or object initializer's existing path.
                let projection = if alias_seed > 0
                    && (projection.len() != alias_seed
                        || origin.file != file
                        || context != Some(parameter.owner)
                        || !live_import_default(&index, parameter, origin.origin))
                {
                    &projection[alias_seed..]
                } else {
                    projection
                };
                parameter
                    .default_mutations
                    .as_ref()
                    .and_then(|mutations| mutations.get(&origin.origin))
                    .is_some_and(|sources| {
                        sources.iter().any(|source| {
                            source.history.invalidated_projected(
                                &source.path,
                                projection,
                                offset,
                                context,
                                origin.file != file,
                            )
                        })
                    })
            })
    }
}

fn live_import_default(
    index: &CallableFileIndex,
    parameter: &ValueParameter,
    origin: usize,
) -> bool {
    origin == 0
        && parameter.projection.is_empty()
        // Imported names use the import-table fallback to scope zero. A local
        // binding (including a same-name shadow) always has an explicit scope.
        && matches!(&parameter.default, ValueExpression::Named { name, scope: None, .. }
            if !name.contains('.') && index.imported.contains_key(name))
}
