impl CallableFileIndex {
    #[inline(never)]
    fn resolve_this_member(
        &self,
        caller: Option<&str>,
        caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
        callee: &str,
        invocation: InvocationKind,
    ) -> Option<ResolvedLocalCallee> {
        if invocation != InvocationKind::Call {
            return None;
        }
        let member = callee.strip_prefix("this.")?;
        if member.is_empty() || member.contains('.') {
            return None;
        }
        let (class, caller_is_static) = self.enclosing_this_class(caller, caller_id)?;
        self.resolve_this_member_on_class(class, member, caller_is_static)
    }

    #[inline(never)]
    fn enclosing_this_class(
        &self,
        caller: Option<&str>,
        caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
    ) -> Option<(&ClassBindingTarget, bool)> {
        let mut scope = caller?;
        loop {
            let (parent, method) = scope.rsplit_once('/')?;
            if self.class_scopes.contains(parent) {
                let class = self.unique_class_binding(parent)?;
                let caller_method = method.split('/').next()?;
                let caller_is_static =
                    self.this_caller_is_static(class, caller_method, caller_id)?;
                return Some((class, caller_is_static));
            }
            scope = parent;
        }
    }

    #[inline(never)]
    fn this_caller_is_static(
        &self,
        class: &ClassBindingTarget,
        caller_method: &str,
        caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
    ) -> Option<bool> {
        let static_id = class
            .static_member_ids
            .get(caller_method)
            .or_else(|| class.static_getter_ids.get(caller_method))
            .or_else(|| class.static_setter_ids.get(caller_method))
            .copied();
        let display = format!("{}/{}", class.scope, caller_method);
        let ids = self.scope_ids_by_display.get(&display);
        if let Some(caller_id) = caller_id {
            if static_id == Some(caller_id) {
                return Some(true);
            }
            if ids.is_some_and(|ids| ids.contains(&caller_id)) {
                return Some(false);
            }
        }
        let ids = ids?;
        (ids.len() == 1).then_some(static_id == Some(ids[0]))
    }

    #[inline(never)]
    fn unique_class_binding(&self, display: &str) -> Option<&ClassBindingTarget> {
        let mut found: Option<&ClassBindingTarget> = None;
        for target in self.class_bindings.values() {
            if target.scope != display {
                continue;
            }
            if found.is_some_and(|existing| existing.class_id != target.class_id) {
                return None;
            }
            found = Some(target);
        }
        found
    }

    #[inline(never)]
    fn unique_class_binding_named(&self, name: &str) -> Option<&ClassBindingTarget> {
        let mut found: Option<&ClassBindingTarget> = None;
        for ((_, binding), target) in &self.class_bindings {
            if binding != name {
                continue;
            }
            if found.is_some_and(|existing| existing.class_id != target.class_id) {
                return None;
            }
            found = Some(target);
        }
        found
    }

    #[inline(never)]
    fn resolve_this_member_on_class<'a>(
        &'a self,
        mut class: &'a ClassBindingTarget,
        member: &str,
        caller_is_static: bool,
    ) -> Option<ResolvedLocalCallee> {
        let mut visited = fx_set();
        loop {
            if !visited.insert(class.class_id) {
                return None;
            }
            if let Some(resolved) = self.this_member_on_own_class(class, member, caller_is_static) {
                return resolved;
            }
            let base = class.local_base.clone()?;
            class = self.unique_class_binding_named(&base)?;
        }
    }

    #[inline(never)]
    fn this_member_on_own_class(
        &self,
        class: &ClassBindingTarget,
        member: &str,
        caller_is_static: bool,
    ) -> Option<Option<ResolvedLocalCallee>> {
        if caller_is_static {
            if let Some(id) = class.static_member_ids.get(member) {
                return Some(Some(self.this_member_callee(class, member, *id)));
            }
            return self
                .instance_member_id(class, member)
                .is_some()
                .then_some(None);
        }
        if let Some(id) = self.instance_member_id(class, member) {
            return Some(Some(self.this_member_callee(class, member, id)));
        }
        class
            .static_member_ids
            .contains_key(member)
            .then_some(None)
    }

    #[inline(never)]
    fn this_member_callee(
        &self,
        class: &ClassBindingTarget,
        member: &str,
        callable_id: crate::codebase::dependencies::extract::CallableId,
    ) -> ResolvedLocalCallee {
        ResolvedLocalCallee {
            callee: format!("{}/{}", class.scope, member),
            callable_id: Some(callable_id),
        }
    }

    #[inline(never)]
    fn instance_member_id(
        &self,
        class: &ClassBindingTarget,
        member: &str,
    ) -> Option<crate::codebase::dependencies::extract::CallableId> {
        let display = format!("{}/{}", class.scope, member);
        let ids = self.scope_ids_by_display.get(&display)?;
        let static_id = class.static_member_ids.get(member);
        let mut found = None;
        for id in ids {
            if static_id == Some(id) {
                continue;
            }
            if found.is_some() {
                return None;
            }
            found = Some(*id);
        }
        found
    }
}
