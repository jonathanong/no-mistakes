use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn replacement_value(
        &mut self,
        path: &std::path::Path,
        name: &str,
        scope: Option<usize>,
        offset: u32,
        context: Option<CallableId>,
    ) -> Option<FlowValue> {
        let (base, property) = name.split_once('.')?;
        if property.contains('.') || scope.is_some_and(|scope| scope != 0) {
            return None;
        }
        let (offset, context) = self.replacement_coordinates(path, offset, context)?;
        let target = self
            .replacements?
            .file(path)?
            .target(base, property, offset, context)?;
        self.replacement_read = true;
        Some(FlowValue::Target(target.clone()))
    }
    pub(super) fn replacement_member(
        &mut self,
        path: &std::path::Path,
        base: &ValueExpression,
        property: &str,
    ) -> Option<FlowValue> {
        let ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } = base
        else {
            return None;
        };
        if name.contains('.') {
            return None;
        }
        // A copied method acquired once outside a repeat region is already
        // checked by the ordinary binding/receiver path at its source read.
        // The replacement catalog tracks repeated writes and would otherwise
        // turn that earlier snapshot into a live, unknown method value.
        if !self.foreign_acquisition
            && self.member_initializer.is_some_and(|(file, start, owner, repeated)| {
                file == path && start == *offset && owner == *context && !repeated
            })
            && context.is_some_and(|owner| {
                matches!(
                    self.source_execution_phase(path, owner),
                    execution_quality::ExecutionPhase::Exact(_)
                )
            })
            && self.replacements.is_some_and(|catalog| {
                catalog.file(path).is_some_and(|file| {
                    !file.acquisition_repeats(*offset, *context)
                })
            })
        {
            return None;
        }
        // A direct argument evaluates within this inline call, before a later
        // write in the same one-shot local owner. Let the ordinary member read
        // enforce that source order instead of projecting a repeated write.
        if !self.foreign_acquisition
            && self.member_initializer.is_none()
            && self.invocation_read.as_ref().is_some_and(|read| {
                read.file == path
                    && read.owner.0 == path
                    && read.context == *context
                    && !read.repeated
                    && self.invocation_start.is_some_and(|start| start <= *offset)
                    && *offset <= read.offset
            })
            && context.is_some_and(|owner| {
                matches!(
                    self.source_execution_phase(path, owner),
                    execution_quality::ExecutionPhase::Exact((_, None))
                )
            })
            && self.replacements.is_some_and(|catalog| {
                catalog.file(path).is_some_and(|file| {
                    !file.acquisition_repeats(*offset, *context)
                })
            })
        {
            return None;
        }
        // The collector normally flattens dotted reads into Named. Computed
        // static members retain their own projection and use the same lookup.
        let file = self.replacements?.file(path)?;
        let (offset, context) = self.replacement_coordinates(path, *offset, *context)?;
        if scope.is_some_and(|scope| scope != 0) {
            return None;
        }
        let target = file.target(name, property, offset, context)?;
        self.replacement_read = true;
        Some(FlowValue::Target(target.clone()))
    }
}

/// Only exact acquired callable leaves enter the catalog. Resolve a repository
/// callable's node now, retaining identity even when two selectors name it.
pub(super) fn replacement_candidate(
    flow: &ValueFlowResolution<'_, '_>,
    value: FlowValue,
) -> Option<ResolvedCallCandidate> {
    let (target, exact) = match value {
        FlowValue::Callable { file, scope, id } => (
            ResolvedCallTarget::RepositoryFunction { file, scope },
            Some(id),
        ),
        FlowValue::Target(target) => (target, None),
        _ => return None,
    };
    let node = graph_call_target_node(
        &flow.inputs.interner,
        flow.facts,
        flow.indexes,
        &target,
        exact,
    );
    let target = match target {
        ResolvedCallTarget::Global { name } => KnownCallTarget::Global { name },
        ResolvedCallTarget::ModuleExport {
            specifier,
            export_path,
            repository_target,
            callable_id,
        } => KnownCallTarget::ModuleExport {
            specifier,
            export_path,
            repository_target,
            callable_id,
        },
        ResolvedCallTarget::RepositoryFunction { file, scope } => {
            KnownCallTarget::RepositoryFunction { file, scope }
        }
        _ => return None,
    };
    Some(ResolvedCallCandidate { target, node })
}

pub(super) fn replacement_target(
    candidates: impl IntoIterator<Item = Option<ResolvedCallCandidate>>,
) -> ResolvedCallTarget {
    let mut known = Vec::new();
    let mut unknown = false;
    for candidate in candidates {
        match candidate {
            Some(candidate) => known.push(candidate),
            None => unknown = true,
        }
    }
    if known.is_empty() {
        return ResolvedCallTarget::Unknown;
    }
    ResolvedCallTarget::Alternatives(std::sync::Arc::new(ResolvedCallAlternatives::new(
        known, unknown,
    )))
}

include!("replacement_coordinates.rs");
