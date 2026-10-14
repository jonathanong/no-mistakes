use super::*;
use std::sync::Arc;
pub(super) type Origins = Option<Arc<OriginNode>>;
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct InvocationRead {
    pub(super) file: std::path::PathBuf,
    pub(super) offset: u32,
    pub(super) context: Option<CallableId>,
    pub(super) owner: (std::path::PathBuf, CallableId),
    pub(super) once: bool,
}
pub(super) struct OriginNode {
    pub(super) kind: OriginKind,
    pub(super) height: usize,
    pub(super) has_receivers: bool,
    pub(super) has_receiver_reads: bool,
    pub(super) has_defaults: bool,
    pub(super) has_unsealed_owned_contexts: bool,
}
pub(super) enum OriginKind {
    Source(DefaultOriginUse),
    Receiver(std::path::PathBuf, usize, String),
    ReceiverUse(Origins, ReceiverRead),
    ReceiverSnapshot(Arc<OriginNode>),
    ReceiverAcquisition(Arc<OriginNode>),
    ReceiverContexts(Arc<OriginNode>, Arc<OriginNode>),
    ReceiverOwnedContexts(Arc<OriginNode>, std::path::PathBuf, CallableId),
    InvocationReceiver {
        read: InvocationRead,
        history_file: std::path::PathBuf,
        history: std::sync::Arc<crate::codebase::dependencies::extract::ValueMutationHistory>,
    },
    Union(Arc<OriginNode>, Arc<OriginNode>),
    Project(Arc<OriginNode>, String),
}
pub(super) struct ReceiverRead {
    pub(super) file: std::path::PathBuf,
    pub(super) offset: u32,
    pub(super) context: Option<CallableId>,
    pub(super) invocation: bool,
    pub(super) owned_context: Option<Box<(std::path::PathBuf, CallableId, bool)>>,
}
impl ReceiverRead {
    pub(super) fn owns(&self, file: &std::path::Path, context: Option<CallableId>) -> bool {
        self.owned_context
            .as_deref()
            .is_some_and(|(owner_file, owner, _)| owner_file == file && Some(*owner) == context)
    }
    pub(super) fn preserves(
        &self,
        file: &std::path::Path,
        context: Option<CallableId>,
        acquiring: bool,
    ) -> bool {
        self.owns(file, context)
            && (acquiring
                || self
                    .owned_context
                    .as_deref()
                    .is_some_and(|(_, _, preserve)| *preserve))
    }
}
pub(super) struct DefaultOriginUse {
    pub(super) file: std::path::PathBuf,
    pub(super) scope: usize,
    pub(super) name: String,
    pub(super) origin: usize,
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
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> bool {
        self.default_origins
            .as_ref()
            .is_some_and(|origins| origins.has_defaults)
            && self.origins_invalidated(file, offset, context, false, projection)
    }
    pub(super) fn receiver_origin_invalidated(&self) -> bool {
        if !self
            .default_origins
            .as_ref()
            .is_some_and(|origins| origins.has_receivers)
        {
            return false;
        }
        self.origins_invalidated(std::path::Path::new(""), 0, None, true, None)
    }
}
