use super::*;
use oxc_syntax::symbol::SymbolId;

impl<'a> Projector<'a, '_> {
    pub(super) fn collect_absent_or_non_finite(
        &mut self,
        arguments: &'a [Argument<'a>],
        spread_from: Option<usize>,
    ) -> Vec<(usize, Option<String>)> {
        let mut out = Vec::new();
        let limit = spread_from.unwrap_or(arguments.len()).min(32);
        for (index, argument) in arguments.iter().take(limit).enumerate() {
            let Some(expr) = argument.as_expression() else {
                continue;
            };
            if self.is_absent_or_non_finite(expr, 0) {
                out.push((index, None));
                continue;
            }
            if let Some(keys) = self
                .closed_object_expr(expr, 0)
                .and_then(|object| self.closed_object_absent_keys(object))
            {
                for key in keys {
                    out.push((index, Some(key)));
                }
            }
        }
        out
    }

    fn closed_object_absent_keys(
        &mut self,
        object: &'a ObjectExpression<'a>,
    ) -> Option<Vec<String>> {
        let mut prop_states: FxHashMap<String, bool> = FxHashMap::default();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            let key =
                (property.kind == PropertyKind::Init && !property.method && !property.computed)
                    .then(|| crate::ast::property_key_name(&property.key))
                    .flatten()
                    .filter(|key| key != "__proto__" && key.len() <= 128)?;
            let bad = self.is_absent_or_non_finite(&property.value, 0);
            prop_states.insert(key.into_owned(), bad);
        }
        let mut bad_keys: Vec<String> = prop_states
            .into_iter()
            .filter_map(|(k, bad)| bad.then_some(k))
            .collect();
        bad_keys.sort();
        Some(bad_keys)
    }

    pub(super) fn is_absent_or_non_finite(&mut self, expr: &Expression<'a>, depth: usize) -> bool {
        if depth >= 16 || !self.take() {
            return false;
        }
        match expr.get_inner_expression() {
            Expression::UnaryExpression(unary) => match unary.operator {
                UnaryOperator::Void => true,
                UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus => {
                    matches!(
                        unary.argument.get_inner_expression(),
                        Expression::Identifier(id) if self.is_unshadowed_ident(id, &["Infinity", "NaN"])
                    )
                }
                _ => false,
            },
            Expression::Identifier(id) => {
                if self.is_unshadowed_ident(id, &["undefined", "NaN", "Infinity"]) {
                    return true;
                }
                let Some(symbol) = id
                    .reference_id
                    .get()
                    .and_then(|r| self.semantic.scoping().get_reference(r).symbol_id())
                    .filter(|&symbol| self.immutable_binding(symbol))
                else {
                    return false;
                };
                self.const_init_for_symbol(symbol, id.span.start)
                    .is_some_and(|init| self.is_absent_or_non_finite(init, depth + 1))
            }
            _ => false,
        }
    }

    pub(super) fn closed_object_expr(
        &mut self,
        expr: &'a Expression<'a>,
        depth: usize,
    ) -> Option<&'a ObjectExpression<'a>> {
        if depth >= 16 || !self.take() {
            return None;
        }
        match expr.get_inner_expression() {
            Expression::ObjectExpression(object) if object.properties.len() <= 256 => Some(object),
            Expression::Identifier(id) => {
                let symbol = id
                    .reference_id
                    .get()
                    .and_then(|r| self.semantic.scoping().get_reference(r).symbol_id())?;
                if !self.immutable_binding(symbol) || !self.safe_uses(symbol, depth + 1) {
                    return None;
                }
                let init = self.const_init_for_symbol(symbol, id.span.start)?;
                self.closed_object_expr(init, depth + 1)
            }
            _ => None,
        }
    }

    fn is_unshadowed_ident(&self, id: &IdentifierReference<'_>, names: &[&str]) -> bool {
        names.contains(&id.name.as_str())
            && id.reference_id.get().is_some_and(|r| {
                self.semantic
                    .scoping()
                    .get_reference(r)
                    .symbol_id()
                    .is_none()
            })
    }

    fn const_init_for_symbol(
        &self,
        symbol: SymbolId,
        use_start: u32,
    ) -> Option<&'a Expression<'a>> {
        let AstKind::VariableDeclarator(decl) = self
            .semantic
            .nodes()
            .kind(self.semantic.scoping().symbol_declaration(symbol))
        else {
            return None;
        };
        let BindingPattern::BindingIdentifier(binding) = &decl.id else {
            return None;
        };
        (binding.symbol_id.get() == Some(symbol) && use_start >= decl.span.end)
            .then_some(decl.init.as_ref())
            .flatten()
    }
}
