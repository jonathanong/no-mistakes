use super::super::namespaces::{self, Binding};
use super::*;

impl Builder<'_> {
    pub(super) fn class(&mut self, path: &Path, file: &TsFileFacts, class: &ClassShape) -> NodeId {
        let node = node(path, file, class);
        if !self.visited.insert((path.to_path_buf(), class.id)) {
            return node;
        }
        if !class.has_base {
            self.output.roots.insert(node.clone());
            return node;
        }
        if self.request.facts.get_ts_facts(path).is_some() {
            return node;
        }
        let base = file.function_calls.iter().find(|call| {
            call.is_callback
                && call.invocation == InvocationKind::Construct
                && call.caller_id == Some(class.id)
        });
        let mut global_base = None;
        if let Some(base) = base {
            // The canonical binding scope is decisive: a nearer opaque value
            // must not fall through to an outer class or an imported namesake.
            let namespace_binding = namespaces::binding(file, class, &base.callee);
            let id = match namespace_binding {
                Binding::Class(id) => Some(id),
                Binding::Opaque => None,
                Binding::Absent => base.callee_binding_scope.and_then(|scope| {
                    file.callable_bindings
                        .iter()
                        .find(|(binding_scope, name, _)| {
                            *binding_scope == scope && name == &base.callee
                        })
                        .map(|(_, _, id)| *id)
                }),
            };
            let bound = id.and_then(|id| file.ancestry.classes.iter().find(|class| class.id == id));
            let target = if let Some(bound) = bound {
                Some(self.class(path, file, bound))
            } else if matches!(namespace_binding, Binding::Absent)
                && base.target_identity == CallTargetIdentity::ModuleExport
            {
                self.reference(path, &base.callee)
            } else {
                None
            };
            if let Some(target) = target {
                self.output
                    .targets
                    .insert((path.to_path_buf(), class.id), target);
            } else if matches!(namespace_binding, Binding::Absent)
                && base.target_identity == CallTargetIdentity::Global
                && is_error_global(&base.callee)
            {
                global_base = Some(base.callee.clone());
            }
        }
        self.output.classes.push(ClassDeclaration {
            file: path.to_path_buf(),
            scope: class_scope(file, class).to_string(),
            callable_id: class.id,
            line: 0,
            exported: false,
            ambient: true,
            namespace: None,
            namespace_escaped: false,
            global_base,
            dynamic_base: base.is_some_and(|base| base.callee == "<unknown>"),
        });
        node
    }
}
