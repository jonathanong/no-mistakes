use super::super::PathDefault;
use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

impl<'a> Projector<'a, '_> {
    /// A never-reassigned parameter, including one direct object-pattern property.
    pub(super) fn param_origin(&mut self, id: &IdentifierReference<'a>) -> Option<PathOrigin> {
        let semantic = self.semantic;
        let symbol = self.binding_symbol(id)?;
        if semantic
            .scoping()
            .get_resolved_references(symbol)
            .any(|reference| reference.flags().is_write())
        {
            return None;
        }
        // oxc records the formal parameter as the symbol declaration, not the
        // inner binding identifier, so the property is found by scanning that pattern.
        let mut node = semantic.scoping().symbol_declaration(symbol);
        let mut located = None;
        for _ in 0..8 {
            if let AstKind::FormalParameter(parameter) = semantic.nodes().kind(node) {
                let member = match &parameter.pattern {
                    BindingPattern::BindingIdentifier(binding)
                        if binding.symbol_id.get() == Some(symbol) =>
                    {
                        None
                    }
                    BindingPattern::ObjectPattern(pattern) => {
                        Some(super::patterns::direct_property_member(pattern, symbol)?)
                    }
                    _ => return None,
                };
                located = Some((parameter.span, member, node));
                break;
            }
            let parent = semantic.nodes().parent_id(node);
            if parent == node {
                break;
            }
            node = parent;
        }
        let (parameter_span, member, parameter_node) = located?;
        let list = semantic.nodes().parent_node(parameter_node);
        let list_kind = list.kind().as_formal_parameters()?;
        let index = list_kind
            .items
            .iter()
            .position(|item| item.span == parameter_span)?;
        let owner = semantic.nodes().parent_id(list.id());
        let start = semantic.nodes().kind(owner).span().start;
        let default = self
            .parameter_default(list_kind.items.get(index)?, member.as_deref(), symbol)
            .map(|origin| {
                if origin
                    .atoms()
                    .iter()
                    .all(|atom| atom.param.is_none() && atom.env.is_none())
                {
                    origin
                } else {
                    PathOrigin::unknown()
                }
            });
        Some(PathOrigin {
            param: Some(index),
            param_default: default.map(|value| PathDefault {
                value: Box::new(value),
                nullish: false,
            }),
            param_member: member,
            param_owner: Some(start),
            owner_line: Some(super::super::line_in(semantic.source_text(), start)),
            ..PathOrigin::unknown()
        })
    }

    fn parameter_default(
        &mut self,
        parameter: &'a FormalParameter<'a>,
        member: Option<&str>,
        symbol: oxc_semantic::SymbolId,
    ) -> Option<PathOrigin> {
        let Some(member) = member else {
            return parameter
                .initializer
                .as_ref()
                .map(|value| self.origin(value, 1));
        };
        let BindingPattern::ObjectPattern(pattern) = &parameter.pattern else {
            return None;
        };
        for property in &pattern.properties {
            if property.computed {
                continue;
            }
            let Some((binding, default)) = super::patterns::direct_binding(&property.value) else {
                continue;
            };
            if binding.symbol_id.get() == Some(symbol) {
                if let Some(default) = default {
                    if parameter.initializer.as_ref().is_some_and(|initializer| {
                        !matches!(initializer.get_inner_expression(), Expression::ObjectExpression(object) if object.properties.is_empty())
                    }) {
                        return None;
                    }
                    return Some(self.origin(default, 1));
                }
                break;
            }
        }
        let init = parameter.initializer.as_ref()?;
        self.member_origin(init, member, 1)
    }
}
