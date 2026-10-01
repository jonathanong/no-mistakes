impl ImportCollector {
    /// A call's callee head is a use the graph resolves, except for
    /// `Errors.Dead.bind(..)`: `bind`, `call` and `apply` hand the class itself
    /// on, so the namespace is read as a value. Other static members, such as
    /// the guard `Errors.Dead.is(error)`, leave the class where it is.
    fn mark_call_namespace_head(&mut self, callee: &Expression<'_>) {
        if !hands_on_member_value(callee) {
            self.mark_namespace_head(callee);
        }
    }

    /// Records `identifier` as a value use of the namespace it names. A type
    /// name, a resolved head, and a local that shadows the name are no use.
    fn note_namespace_value_use(&mut self, identifier: &IdentifierReference<'_>) {
        let name = identifier.name.as_str();
        if self.namespace.type_depth > 0
            || !self.namespace.names.contains(name)
            || self.namespace.benign_heads.contains(&identifier.span.start)
            || self
                .local_stack
                .iter()
                .skip(1)
                .any(|scope| scope.contains(name))
        {
            return;
        }
        if let Some(path) = self.namespace_path_of(name) {
            self.namespace.value_uses.insert(path);
        }
    }

    /// What `name` denotes at this point: the nearest declared namespace of
    /// that name, looking outward from the namespace the walk is inside, else
    /// an import. A same-named value that is no namespace is none of them.
    fn namespace_path_of(&self, name: &str) -> Option<String> {
        let declared = &self.namespace.facts.declared;
        for parent in self.namespace.stack.iter().rev() {
            let path = format!("{parent}.{name}");
            if declared.binary_search(&path).is_ok() {
                return Some(path);
            }
        }
        let top_level = declared
            .binary_search_by(|path| path.as_str().cmp(name))
            .is_ok();
        (top_level || self.predeclared_imported_bindings.contains(name)).then(|| name.to_string())
    }

    /// Runs `visit` with identifiers read as type names: `typeof Errors`,
    /// `implements Errors.Marker` and `interface X extends Errors.Y` name the
    /// namespace in erased code only.
    fn walk_as_type_names(&mut self, visit: impl FnOnce(&mut Self)) {
        self.namespace.type_depth += 1;
        visit(self);
        self.namespace.type_depth -= 1;
    }
}

/// `Errors.Dead.bind(..)`, `.call(..)` and `.apply(..)`: the method receives
/// the member itself, so a class it names leaves the graph's sight. A
/// one-segment receiver (`handler.bind(this)`) has no member to hand on.
fn hands_on_member_value(callee: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(callee)
    else {
        return false;
    };
    matches!(member.property.name.as_str(), "bind" | "call" | "apply")
        && matches!(
            crate::codebase::ts_source::unwrap_ts_wrappers(&member.object),
            Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_)
        )
}
