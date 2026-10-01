impl CallSiteResolution<'_, '_> {
    /// The edge and call site for a call whose target is already resolved.
    fn finish(
        &self,
        call: &FunctionCall,
        resolved_target: ResolvedCallTarget,
        callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
    ) -> (Option<Edge>, ResolvedCallSite) {
        let path = self.path;
        let source = call.caller.as_deref().map_or_else(
            || NodeId::file_in(&self.edge_inputs.interner, path),
            |caller| NodeId::scoped_in(&self.edge_inputs.interner, path, caller, call.caller_id),
        );
        let edge = graph_call_target_node(
            &self.edge_inputs.interner,
            self.facts,
            self.indexes,
            &resolved_target,
            callable_id,
        )
        .map(|target| (source, target, EdgeKind::Call));
        (
            edge,
            ResolvedCallSite {
                file: path.to_path_buf(),
                caller: call.caller.clone(),
                caller_id: call.caller_id,
                source_callee: call.callee.clone(),
                line: call.line,
                offset: call.offset,
                invocation: call.invocation,
                target: resolved_target,
            },
        )
    }

    /// A construction naming a class of a namespace this file declares:
    /// `new Errors.X()`, or a bare `new X()` inside the namespace body that
    /// declares `X`. The class's exact id keeps same-named classes of
    /// different namespaces apart. Dotted names are left to the regular
    /// resolver when it already knows what they are, such as a class static. A
    /// name bound in a nested scope (a parameter or local named `Errors`) is
    /// not the namespace; only the program scope `0` and the namespace bodies
    /// bind names a namespace lookup may resolve.
    fn namespace_member(
        &self,
        call: &FunctionCall,
        identity: CallTargetIdentity,
    ) -> Option<(
        ResolvedCallTarget,
        crate::codebase::dependencies::extract::CallableId,
    )> {
        let table = &self.index.namespaces;
        let known = !matches!(
            identity,
            CallTargetIdentity::Unknown | CallTargetIdentity::Global
        );
        if call.invocation != InvocationKind::Construct
            || table.is_empty()
            || (call.callee.contains('.') && known)
            || !table.binds_namespace_names(call.callee_binding_scope)
        {
            return None;
        }
        let context = table.context(call.caller_id, call.offset);
        match table.lookup(context, &call.callee) {
            NamespaceLookup::Member { scope, id } => Some((
                ResolvedCallTarget::RepositoryFunction {
                    file: self.path.to_path_buf(),
                    scope: scope.to_string(),
                },
                id,
            )),
            NamespaceLookup::Missing { root } => {
                self.escape(self.path, root);
                None
            }
            NamespaceLookup::Absent => None,
        }
    }

    /// Records that a use of `root` in `file` (`"*"`: every root there) cannot
    /// be followed to a class, so none of its classes may be reported.
    fn escape(&self, file: &std::path::Path, root: &str) {
        self.escapes
            .borrow_mut()
            .insert((file.to_path_buf(), root.to_string()));
    }
}
