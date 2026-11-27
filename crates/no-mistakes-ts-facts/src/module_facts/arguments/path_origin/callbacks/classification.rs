use super::*;
use oxc_semantic::SymbolId;

impl<'a> Projector<'a, '_> {
    pub(super) fn callback_temp_argument(
        &mut self,
        kind: AstKind<'a>,
        position: usize,
        index: usize,
    ) -> bool {
        let params = match kind {
            AstKind::Function(function) => &function.params,
            AstKind::VariableDeclarator(declaration) => match declaration
                .init
                .as_ref()
                .map(Expression::get_inner_expression)
            {
                Some(Expression::ArrowFunctionExpression(function)) => &function.params,
                Some(Expression::FunctionExpression(function)) => &function.params,
                _ => return false,
            },
            _ => return false,
        };
        let Some(parameter) = params.items.get(position) else {
            return false;
        };
        let BindingPattern::BindingIdentifier(binding) = &parameter.pattern else {
            return false;
        };
        binding
            .symbol_id
            .get()
            .is_some_and(|symbol| self.callback_symbol_temp(symbol, index, 0))
    }

    fn callback_symbol_temp(&mut self, symbol: SymbolId, index: usize, depth: usize) -> bool {
        if depth >= 8 {
            return true;
        }
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() {
                return true;
            }
            if reference.is_type() {
                continue;
            }
            let node = reference.node_id();
            match semantic.nodes().parent_kind(node) {
                AstKind::CallExpression(call)
                    if call.callee.span() == semantic.nodes().kind(node).span() =>
                {
                    let Some(argument) =
                        call.arguments.get(index).and_then(Argument::as_expression)
                    else {
                        continue;
                    };
                    if self.origin(argument, 1).anchor == PathAnchor::Temp {
                        return true;
                    }
                }
                AstKind::VariableDeclarator(declaration) => {
                    let BindingPattern::BindingIdentifier(alias) = &declaration.id else {
                        continue;
                    };
                    if alias
                        .symbol_id
                        .get()
                        .is_some_and(|symbol| self.callback_symbol_temp(symbol, index, depth + 1))
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
#[path = "classification_tests.rs"]
mod tests;
