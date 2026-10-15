use super::*;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

type Name = (std::path::PathBuf, usize, String);
type Call = (std::path::PathBuf, u32, u32);
#[derive(Clone)]
struct OriginIdentity(Arc<OriginNode>);
impl PartialEq for OriginIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for OriginIdentity {}
impl Hash for OriginIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct BindingRead {
    pub(super) source: execution_quality::Location,
    pub(super) location: execution_quality::Location,
    pub(super) available_at: u32,
    pub(super) read_at: u32,
    pub(super) read_context: Option<CallableId>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct NullishKey {
    binding: (std::path::PathBuf, usize, String, u32, Option<CallableId>),
    read: BindingRead,
    invocation: Option<default_origins::InvocationRead>,
    parameter_frame: Option<ParameterKey>,
    invocation_start: Option<u32>,
    executing: Option<(
        std::path::PathBuf,
        CallableId,
        execution_quality::ExecutionPhase,
    )>,
    initializing: Vec<(std::path::PathBuf, CallableId, usize)>,
    snapshot: Option<(std::path::PathBuf, u32)>,
    member_initializer: Option<(std::path::PathBuf, u32, Option<CallableId>, bool)>,
    member_read: Option<(std::path::PathBuf, u32, Option<CallableId>)>,
    acquisition: Option<OriginIdentity>,
    projection: Vec<String>,
    names: Vec<Name>,
    calls: Vec<Call>,
    parameters_enabled: bool,
    foreign_acquisition: bool,
}
impl ValueFlowResolution<'_, '_> {
    pub(super) fn nullish_key(
        &self,
        path: &std::path::Path,
        binding: &ValueBinding,
        read: BindingRead,
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> NullishKey {
        // Immutable binding guards have already checked every incoming read.
        // Initializers retain their authored reads, rather than caller offsets.
        #[cfg(feature = "test-instrumentation")]
        self.cache.record_nullish_work(
            3,
            self.visiting.len() + self.calls_visiting.len() + self.initializing.len(),
        );
        let mut names = self
            .visiting
            .iter()
            .filter(|(file, scope, name)| {
                self.indexes
                    .file(self.facts, file)
                    .is_none_or(|index| scoped_name(&index.values.bindings, *scope, name).is_none())
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut calls = self.calls_visiting.iter().cloned().collect::<Vec<_>>();
        names.sort_unstable();
        calls.sort_unstable();
        NullishKey {
            binding: (
                path.to_path_buf(),
                binding.scope,
                binding.name.clone(),
                binding.declared_at,
                binding.owner,
            ),
            read,
            invocation: self.invocation_read.clone(),
            parameter_frame: self.parameter_frame.cloned(),
            invocation_start: self.invocation_start,
            executing: self
                .executing_read
                .map(|(file, owner, phase)| (file.to_path_buf(), owner, phase)),
            initializing: self.initializing.clone(),
            snapshot: self.module_snapshot.clone(),
            member_initializer: self
                .member_initializer
                .map(|(file, offset, owner, repeated)| {
                    (file.to_path_buf(), offset, owner, repeated)
                }),
            member_read: self.member_read.clone(),
            acquisition: self.acquisition_context.clone().map(OriginIdentity),
            projection: projection.map_or_else(Vec::new, |projection| {
                projection
                    .members()
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            }),
            names,
            calls,
            parameters_enabled: self.parameters_enabled,
            foreign_acquisition: self.foreign_acquisition,
        }
    }
}
