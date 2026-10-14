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
    ) -> bool {
        self.indexes
            .file(self.facts, &origin.file)
            .is_some_and(|index| {
                scoped_name(&index.values.parameters, origin.scope, &origin.name)
                    .and_then(|parameter| parameter.default_mutations.as_ref())
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
