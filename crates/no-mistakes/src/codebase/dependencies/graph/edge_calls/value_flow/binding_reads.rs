use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn acquired_binding_member(
        binding: Option<&ValueBinding>,
        member: Option<&str>,
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> bool {
        projection.is_none()
            && member.is_none()
            && binding.is_some_and(|binding| matches!(binding.value, ValueExpression::Member(..)))
    }

    pub(super) fn binding_value(
        &mut self,
        path: &std::path::Path,
        binding: &ValueBinding,
        source: (u32, Option<CallableId>),
        location: (u32, Option<CallableId>),
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> FlowValue {
        let Some((available_at, read_at, read_context)) = self.binding_read(
            path,
            binding.owner,
            source.0,
            source.1,
            location.0,
            location.1,
        ) else {
            return FlowValue::Unknown;
        };
        let repeated_capture = source.1.is_some()
            && binding.owner != source.1
            && self
                .expression_phase(path, source.0, source.1)
                .is_repeated();
        if binding
            .mutation_contexts
            .iter()
            .any(|writer| *writer != read_context)
            || available_at < binding.declared_at
            || binding
                .invalidated_at
                .is_some_and(|cutoff| repeated_capture || read_at >= cutoff)
        {
            return FlowValue::Unknown;
        }
        // Whole-object aliases keep live descendants; copied member initializers
        // acquire at their authored source, outside the repeated body read.
        let projection = projection.filter(|_| {
            matches!(&binding.value, ValueExpression::Named { name, .. } if !name.contains('.'))
        });
        let receiver_writers = self.indexes.file(self.facts, path).is_some_and(|index| {
            index
                .values
                .receiver_mutations
                .as_ref()
                .is_some_and(|receivers| !receivers.writer_contexts.is_empty())
        });
        let previous = self.module_snapshot.take();
        // Only a module binding's own copied-member initializer is acquired once.
        // Live object aliases and callable-local acquisitions retain their timing.
        self.module_snapshot = (receiver_writers
            && binding.scope == 0
            && binding.owner.is_none()
            && matches!(binding.value, ValueExpression::Member(..))
            && Self::member_read_location(&binding.value)
                .is_some_and(|(_, owner)| owner.is_none()))
        .then(|| {
            (
                path.to_path_buf(),
                Self::member_read_location(&binding.value)
                    .expect("qualified static member initializer")
                    .0,
            )
        });
        let previous_initializer = self.member_initializer.take();
        self.member_initializer = ((binding.repeated || self.replacements.is_some())
            && matches!(binding.value, ValueExpression::Member(..)))
        .then(|| {
            let (offset, context) = Self::member_read_location(&binding.value)?;
            Some((
                self.inputs.graph_files.visible_path(path)?,
                offset,
                context,
                binding.repeated,
            ))
        })
        .flatten();
        let value = self.memoized_nullish_binding(
            path,
            binding,
            nullish_key::BindingRead {
                source,
                location,
                available_at,
                read_at,
                read_context,
            },
            projection,
        );
        self.member_initializer = previous_initializer;
        self.module_snapshot = previous;
        value
    }

    /// Captured bindings must exist and remain unchanged at execution.
    /// Local TDZ checks and stored initializer acquisition keep source offsets.
    pub(super) fn binding_read(
        &self,
        path: &std::path::Path,
        declared_owner: Option<CallableId>,
        source_offset: u32,
        source_context: Option<CallableId>,
        offset: u32,
        context: Option<CallableId>,
    ) -> Option<(u32, u32, Option<CallableId>)> {
        let Some(read) = &self.invocation_read else {
            return Some((source_offset, offset, context));
        };
        if read.file != path || source_context.is_none() || declared_owner == source_context {
            return Some((source_offset, offset, context));
        }
        if let Some((_, _, phase)) = self
            .executing_read
            .filter(|(file, owner, _)| *file == path && Some(*owner) == source_context)
        {
            return phase
                .location()
                .map(|(read_at, read_context)| (read_at, read_at, read_context));
        }
        Some(
            self.acquired_binding_read(path, source_context.expect("captured binding"))
                .unwrap_or((read.offset, read.offset, read.context)),
        )
    }

    /// Lazy returned fields borrow their existing owner-qualified acquisition
    /// frames. All callers must see initialization and precede later mutation.
    fn acquired_binding_read(
        &self,
        path: &std::path::Path,
        owner: CallableId,
    ) -> Option<(u32, u32, Option<CallableId>)> {
        let root = self.acquisition_context.as_ref()?;
        let mut available_at = u32::MAX;
        let mut read_at = 0;
        let mut context = None;
        for frame in receiver_owned_contexts::frames(root, path, owner) {
            let Some(OriginNode {
                kind: OriginKind::ReceiverUse(_, read),
                ..
            }) = frame
            else {
                return Some((0, u32::MAX, None));
            };
            if read.repeated
                || read.file != path
                || context.is_some_and(|prior| prior != read.context)
            {
                return Some((0, u32::MAX, None));
            }
            available_at = available_at.min(read.offset);
            read_at = read_at.max(read.offset);
            context = Some(read.context);
        }
        context.map(|context| (available_at, read_at, context))
    }
    pub(super) fn imported_value(&self, file: &CallableFileIndex, base: &str) -> FlowValue {
        if let Some(binding) = file.imported.get(base).filter(|binding| {
            binding.kind == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
        }) {
            FlowValue::Target(ResolvedCallTarget::ModuleExport {
                specifier: binding.specifier.clone(),
                export_path: String::new(),
                repository_target: None,
                callable_id: None,
            })
        } else {
            FlowValue::Target(
                module_export_target(file, base, None, None).unwrap_or(ResolvedCallTarget::Unknown),
            )
        }
    }
}
