use super::*;
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct DefaultOriginUse {
    file: std::path::PathBuf,
    scope: usize,
    name: String,
    origin: usize,
    projection: Vec<String>,
}
pub(super) fn merge(
    target: &mut Option<FxHashSet<DefaultOriginUse>>,
    source: Option<FxHashSet<DefaultOriginUse>>,
) {
    if let Some(source) = source {
        if let Some(target) = target {
            target.extend(source);
        } else {
            *target = Some(source);
        }
    }
}
impl ValueFlowResolution<'_, '_> {
    pub(super) fn extend_default_origins(&mut self, origins: Option<&FxHashSet<DefaultOriginUse>>) {
        if let Some(origins) = origins {
            self.default_origins
                .get_or_insert_with(fx_set)
                .extend(origins.iter().cloned());
        }
    }
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
            self.default_origins
                .get_or_insert_with(fx_set)
                .insert(DefaultOriginUse {
                    file: file.to_path_buf(),
                    scope: parameter.scope,
                    name: parameter.name.clone(),
                    origin,
                    projection: Vec::new(),
                });
        }
    }
    pub(super) fn project_default_origins(&mut self, member: &str) {
        if let Some(origins) = self.default_origins.take() {
            self.default_origins = Some(
                origins
                    .into_iter()
                    .map(|mut origin| {
                        origin.projection.push(member.to_string());
                        origin
                    })
                    .collect(),
            );
        }
    }
    pub(super) fn default_origin_invalidated(
        &self,
        file: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> bool {
        self.default_origins
            .as_ref()
            .into_iter()
            .flatten()
            .any(|origin| {
                self.indexes
                    .file(self.facts, &origin.file)
                    .is_some_and(|index| {
                        scoped_name(&index.values.parameters, origin.scope, &origin.name)
                            .and_then(|parameter| parameter.default_mutations.as_ref())
                            .and_then(|mutations| mutations.get(&origin.origin))
                            .is_some_and(|paths| {
                                paths.iter().any(|(receiver, mutation)| {
                                    receiver.len() >= origin.projection.len()
                                        && receiver.iter().zip(&origin.projection).all(
                                            |(actual, expected)| {
                                                actual == expected || actual == "*"
                                            },
                                        )
                                        && (origin.file != file
                                            || mutation.offset <= offset
                                            || mutation
                                                .contexts
                                                .iter()
                                                .any(|writer| *writer != context))
                                })
                            })
                    })
            })
    }
}
