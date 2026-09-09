impl CallableFileIndex {
    fn resolve_this_member(
        &self,
        caller: Option<&str>,
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
        let (class, caller_is_static) = self.enclosing_this_class(caller)?;
        self.resolve_this_member_on_class(class, member, caller_is_static)
    }

    fn enclosing_this_class(&self, caller: Option<&str>) -> Option<(&ClassBindingTarget, bool)> {
        let mut scope = caller?;
        loop {
            let (parent, method) = scope.rsplit_once('/')?;
            if self.class_scopes.contains(parent) {
                let class = self.unique_class_binding(parent)?;
                let caller_method = method.split('/').next()?;
                let caller_is_static = class.static_member_ids.contains_key(caller_method)
                    || class.static_getter_ids.contains_key(caller_method)
                    || class.static_setter_ids.contains_key(caller_method);
                return Some((class, caller_is_static));
            }
            scope = parent;
        }
    }

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

    fn instance_member_id(
        &self,
        class: &ClassBindingTarget,
        member: &str,
    ) -> Option<crate::codebase::dependencies::extract::CallableId> {
        if class.static_member_ids.contains_key(member) {
            return None;
        }
        let display = format!("{}/{}", class.scope, member);
        let ids = self.scope_ids_by_display.get(&display)?;
        (ids.len() == 1).then_some(ids[0])
    }
}
