use super::{PathAnchor, PathOrigin, PathValueKind, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

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
                let mut origin = self.origin(init, depth + 1);
                if origin.getter.is_some()
                    || (!origin.alternatives.is_empty()
                        && origin
                            .atoms()
                            .iter()
                            .any(|atom| atom.env.is_some() || atom.getter.is_some()))
                    || (origin.anchor == PathAnchor::SharedTemp && origin.env.is_some())
                {
                    return captured_unknown(&origin);
                }
                if origin.env.is_some() && origin.env_capture.is_none() {
                    let symbol = self.binding_symbol(id).unwrap();
                    let node = self.semantic.scoping().symbol_declaration(symbol);
                    if self.semantic.nodes().ancestors(node).any(|node| {
                        matches!(
                            node.kind(),
                            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                        )
                    }) {
                        return captured_unknown(&origin);
                    }
                    origin.env_capture = Some(init.span().start);
                    if let Some(hint) = &mut origin.env_diagnostic {
                        hint.captured = true;
                    }
                }
                self.value_requirement(id, origin)
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

fn captured_unknown(origin: &PathOrigin) -> PathOrigin {
    let mut unknown = PathOrigin::unknown();
    if let [atom] = origin.atoms() {
        unknown.env_diagnostic = atom.env_diagnostic.clone();
        if let Some(hint) = &mut unknown.env_diagnostic {
            hint.captured = true;
        }
        unknown.captured_getter = atom.getter.or(atom.captured_getter);
    }
    unknown
}
