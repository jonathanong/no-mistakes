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
        hint_len: usize,
        file: &std::path::Path,
        read: (u32, Option<CallableId>),
        reader: &impl Fn(Option<CallableId>) -> receiver_reads::ReadOffset,
    ) -> bool {
        self.indexes
            .file(self.facts, &origin.file)
            .is_some_and(|index| {
                scoped_name(&index.values.parameters, origin.scope, &origin.name)
                    .and_then(|parameter| {
                        parameter
                            .default_mutations
                            .as_ref()?
                            .get(&origin.origin)
                            .map(|sources| (parameter, sources))
                    })
                    .is_some_and(|(parameter, sources)| {
                        sources.iter().any(|source| {
                            // Existing origin projections already describe an
                            // acquired segment; do not apply its use hint twice.
                            if hint_len > 0
                                && projection.len() == hint_len
                                && source.history.has_assignments
                                && !source.history.has_unassigned
                                && source.path.iter().all(Option::is_some)
                            {
                                let path = projection
                                    .iter()
                                    .copied()
                                    .chain(source.path.iter().rev().filter_map(Option::as_deref))
                                    .collect::<Vec<_>>();
                                // Only the owning synchronous body may use its
                                // acquisition timing for outer-context writes.
                                // Forwarded and captured bodies retain the lexical guard.
                                let acquired = read.1 == Some(parameter.owner)
                                    && self.synchronous_callable(&origin.file, parameter.owner);
                                source
                                    .history
                                    .assignment_invalidated_reads(&path, &|writer| {
                                        if origin.file != file {
                                            None
                                        } else if acquired {
                                            reader(writer).known()
                                        } else {
                                            match reader(writer) {
                                                receiver_reads::ReadOffset::Repeated => None,
                                                _ => (writer == read.1).then_some(read.0),
                                            }
                                        }
                                    })
                            } else {
                                source.history.invalidated_projected(
                                    &source.path,
                                    &projection[hint_len..],
                                    read.0,
                                    read.1,
                                    origin.file != file
                                        || matches!(
                                            reader(read.1),
                                            receiver_reads::ReadOffset::Repeated
                                        ),
                                )
                            }
                        })
                    })
            })
    }
}
