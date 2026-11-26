use super::super::PathDefault;
use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

impl<'a> Projector<'a, '_> {
    pub(super) fn spread_excludes(&mut self, expression: &Expression<'a>, key: &str) -> bool {
        if !self.take() {
            return false;
        }
        let Expression::Identifier(id) = expression.get_inner_expression() else {
            return false;
        };
        let Some(symbol) = self.binding_symbol(id) else {
            return false;
        };
        if !self.safe_uses(symbol, 0) {
            return false;
        }
        let semantic = self.semantic;
        let pattern = match semantic
            .nodes()
            .kind(semantic.scoping().symbol_declaration(symbol))
        {
            AstKind::FormalParameter(parameter) => &parameter.pattern,
            AstKind::VariableDeclarator(declaration) => &declaration.id,
            _ => return false,
        };
        let BindingPattern::ObjectPattern(pattern) = pattern else {
            return false;
        };
        let Some(rest) = &pattern.rest else {
            return false;
        };
        if !matches!(&rest.argument, BindingPattern::BindingIdentifier(binding) if binding.symbol_id.get() == Some(symbol))
        {
            return false;
        }
        pattern.properties.iter().any(|property| {
            !property.computed
                && crate::ast::property_key_name(&property.key).as_deref() == Some(key)
        })
    }
}

impl<'a> Projector<'a, '_> {
    pub(super) fn rest_member_origin(
        &mut self,
        id: &IdentifierReference<'a>,
        key: &str,
        depth: usize,
    ) -> Option<PathOrigin> {
        if depth >= 16 || !self.take() {
            return None;
        }
        let semantic = self.semantic;
        let symbol = self.binding_symbol(id)?;
        let declaration = semantic.scoping().symbol_declaration(symbol);
        let (pattern, parameter) = match semantic.nodes().kind(declaration) {
            AstKind::FormalParameter(parameter) => (&parameter.pattern, Some(parameter)),
            AstKind::VariableDeclarator(declaration) => (&declaration.id, None),
            _ => return None,
        };
        let BindingPattern::ObjectPattern(pattern) = pattern else {
            return None;
        };
        let rest = pattern.rest.as_ref()?;
        if !matches!(&rest.argument, BindingPattern::BindingIdentifier(binding) if binding.symbol_id.get() == Some(symbol))
        {
            return None;
        }
        if pattern.properties.iter().any(|property| {
            property.computed
                || crate::ast::property_key_name(&property.key).as_deref() == Some(key)
        }) {
            return None;
        }
        if let Some(parameter) = parameter {
            let list = semantic.nodes().parent_node(declaration);
            let params = list.kind().as_formal_parameters()?;
            let index = params
                .items
                .iter()
                .position(|item| item.span == parameter.span)?;
            let start = semantic.nodes().parent_kind(list.id()).span().start;
            let default = parameter.initializer.as_ref().filter(|value| {
                !matches!(value.get_inner_expression(), Expression::ObjectExpression(object) if object.properties.is_empty())
            }).map(|_| PathDefault { value: Box::new(PathOrigin::unknown()), nullish: false });
            return Some(PathOrigin {
                param: Some(index),
                param_member: Some(key.to_owned()),
                param_owner: Some(start),
                owner_line: Some(super::super::line_in(semantic.source_text(), start)),
                param_default: default,
                ..PathOrigin::unknown()
            });
        }
        let AstKind::VariableDeclarator(declaration) = semantic.nodes().kind(declaration) else {
            return None;
        };
        let init = declaration.init.as_ref()?;
        if let Expression::Identifier(source) = init.get_inner_expression() {
            if !self.destructure_source_stable(source) {
                return None;
            }
            let mut origin = self.origin(init, depth + 1);
            if origin.param.is_some() && origin.param_member.is_none() {
                origin.param_member = Some(key.to_owned());
                return Some(origin);
            }
        }
        self.member_origin(init, key, depth + 1)
    }
}
