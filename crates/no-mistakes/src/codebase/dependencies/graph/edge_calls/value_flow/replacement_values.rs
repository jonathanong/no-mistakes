use super::*;

impl ValueFlowResolution<'_, '_> {
    /// Resolve acquisition at its source position, before receiver-origin
    /// invalidation turns the live receiver into an unknown method value.
    fn replacement_coordinates(
        &self,
        path: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> (u32, Option<CallableId>) {
        if let Some(read) = self.invocation_read.as_ref().filter(|read| {
            read.file == path && read.owner.0 == path && context == Some(read.owner.1)
        }) {
            return (read.offset, read.context);
        }
        // Pure synchronous method-return helpers acquire the method at their
        // physical invocation. Borrow that already prepared frame directly;
        // no read-local frames, sets, or retained origin traversal are needed.
        if let Some(owner) = context {
            if self
                .replacements
                .and_then(|catalog| catalog.file(path))
                .is_some_and(|file| file.snapshot_owner(owner))
            {
                if let Some(OriginKind::ReceiverUse(_, read)) =
                    self.acquisition_context.as_ref().map(|root| &root.kind)
                {
                    if read.file == path && read.owns(path, context) {
                        return (read.offset, read.context);
                    }
                }
            }
        }
        (offset, context)
    }
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
        let (offset, context) = self.replacement_coordinates(path, offset, context);
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
        // The collector normally flattens dotted reads into Named. Computed
        // static members retain their own projection and use the same lookup.
        let file = self.replacements?.file(path)?;
        let (offset, context) = self.replacement_coordinates(path, *offset, *context);
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
