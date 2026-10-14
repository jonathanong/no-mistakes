use super::*;
use std::sync::Arc;
pub(super) type Origins = Option<Arc<OriginNode>>;
pub(super) struct OriginNode {
    pub(super) kind: OriginKind,
    pub(super) height: usize,
}
pub(super) enum OriginKind {
    Source(DefaultOriginUse),
    Union(Arc<OriginNode>, Arc<OriginNode>),
    Project(Arc<OriginNode>, String),
}
pub(super) struct DefaultOriginUse {
    file: std::path::PathBuf,
    scope: usize,
    name: String,
    origin: usize,
}
pub(super) fn merge(target: &mut Origins, source: Origins, cache: &ParameterCache) {
    if let Some(source) = source {
        *target = Some(match target.take() {
            Some(target) => origin_tree::join(target, source, cache),
            None => source,
        });
    }
}
impl ValueFlowResolution<'_, '_> {
    pub(super) fn extend_default_origins(&mut self, origins: Option<&Arc<OriginNode>>) {
        merge(&mut self.default_origins, origins.cloned(), self.cache);
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
    pub(super) fn project_default_origins(&mut self, member: &str) {
        if let Some(origins) = self.default_origins.take() {
            self.default_origins = Some(
                self.cache
                    .origin_node(OriginKind::Project(origins, member.to_string()), 1),
            );
        }
    }
    pub(super) fn default_origin_invalidated(
        &self,
        file: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> bool {
        let Some(root) = &self.default_origins else {
            return false;
        };
        enum Step<'a> {
            Visit(&'a OriginNode),
            Pop,
        }
        let mut pending = vec![Step::Visit(root)];
        let mut projection = Vec::new();
        let mut visited = fx_set();
        while let Some(step) = pending.pop() {
            let Step::Visit(node) = step else {
                projection.pop();
                continue;
            };
            if !visited.insert((node as *const OriginNode as usize, projection.clone())) {
                continue;
            }
            match &node.kind {
                OriginKind::Union(left, right) => {
                    pending.push(Step::Visit(right));
                    pending.push(Step::Visit(left));
                }
                OriginKind::Project(source, member) => {
                    projection.push(member.as_str());
                    pending.push(Step::Pop);
                    pending.push(Step::Visit(source));
                }
                OriginKind::Source(origin) => {
                    if self.source_origin_invalidated(origin, &projection, file, offset, context) {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn source_origin_invalidated(
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
