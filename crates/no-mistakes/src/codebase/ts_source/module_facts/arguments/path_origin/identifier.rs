use super::{PathAnchor, PathOrigin, PathValueKind, Projector};
use oxc_ast::ast::IdentifierReference;

impl<'a> Projector<'a, '_> {
    pub(super) fn identifier(&mut self, id: &IdentifierReference<'a>, depth: usize) -> PathOrigin {
        if id.name == "undefined" && self.binding_symbol(id).is_none() {
            return PathOrigin {
                nullish: true,
                value_kind: PathValueKind::Scalar,
                ..PathOrigin::unknown()
            };
        }
        let global = matches!(id.name.as_str(), "__dirname" | "__filename");
        if global && self.binding_symbol(id).is_none() {
            return PathOrigin::anchored(PathAnchor::Module);
        }
        match self.const_init(id) {
            Some(init) => {
                let origin = self.origin(init, depth + 1);
                // A captured env read ran when the alias was initialised, not at the
                // call, so a later owned write does not cover it.
                if origin
                    .atoms()
                    .iter()
                    .any(|atom| atom.env.is_some() || atom.getter.is_some())
                {
                    let mut unknown = PathOrigin::unknown();
                    unknown.env_diagnostic = origin
                        .atoms()
                        .iter()
                        .find_map(|atom| atom.env_diagnostic.clone());
                    if let Some(hint) = &mut unknown.env_diagnostic {
                        hint.captured = true;
                    }
                    unknown.captured_getter = origin
                        .atoms()
                        .iter()
                        .find_map(|atom| atom.getter.or(atom.captured_getter));
                    unknown
                } else {
                    self.value_requirement(id, origin)
                }
            }
            None => {
                let mut origin = self
                    .destructured_origin(id, depth)
                    .or_else(|| self.param_origin(id))
                    .or_else(|| self.owned_binding(id, depth))
                    .unwrap_or_else(PathOrigin::unknown);
                if origin.param.is_none()
                    && origin.env.is_none()
                    && origin.anchor == PathAnchor::Unknown
                {
                    origin.reassigned_at = self.reassigned_parameter(id);
                }
                self.value_requirement(id, origin)
            }
        }
    }
}
