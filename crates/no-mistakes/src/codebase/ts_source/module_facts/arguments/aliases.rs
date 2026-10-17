use super::*;
use oxc_span::GetSpan;
use oxc_syntax::symbol::SymbolId;

#[cfg_attr(test, path = "alias_safety_tests.rs")]
#[cfg_attr(not(test), path = "safety.rs")]
#[macro_use]
mod safety;
pub(super) use safety::AliasSafety;

impl Projector<'_, '_> {
    pub(super) fn named(
        &mut self,
        identifier: &IdentifierReference<'_>,
        depth: usize,
    ) -> ArgumentValue {
        let semantic = self.semantic;
        let Some(reference) = identifier.reference_id.get() else {
            return ArgumentValue::Unknown;
        };
        let Some(symbol) = semantic.scoping().get_reference(reference).symbol_id() else {
            return ArgumentValue::Unknown;
        };
        if !self.immutable_binding(symbol) {
            return ArgumentValue::Unknown;
        }
        let AstKind::VariableDeclarator(declaration) = semantic
            .nodes()
            .kind(semantic.scoping().symbol_declaration(symbol))
        else {
            return ArgumentValue::Unknown;
        };
        let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
            return ArgumentValue::Unknown;
        };
        if binding.symbol_id.get() != Some(symbol) || identifier.span.start < declaration.span.end {
            return ArgumentValue::Unknown;
        }
        let Some(initializer) = &declaration.init else {
            return ArgumentValue::Unknown;
        };
        let value = self.value(initializer, depth + 1);
        if matches!(value, ArgumentValue::Array(_) | ArgumentValue::Object(_))
            && !self.safe_uses(symbol, depth + 1)
        {
            return ArgumentValue::Unknown;
        }
        value
    }

    // Cache failures too: a popular alias never repeats an unbounded reference scan.
    fn immutable_binding(&mut self, symbol: SymbolId) -> bool {
        if let Some(&known) = self.immutable.entries.get(&symbol) {
            return known;
        }
        let mut known = self
            .semantic
            .scoping()
            .symbol_flags(symbol)
            .is_const_variable();
        if known {
            let mut references = self.semantic.scoping().get_resolved_references(symbol);
            for _ in 0..256 {
                let Some(reference) = references.next() else {
                    self.immutable.entries.insert(symbol, true);
                    return true;
                };
                record_reference_scan!(self.immutable);
                if reference.flags().is_write() {
                    break;
                }
            }
            // Exhaustion is unknown, including exactly-at-limit aliases.
            known = false;
        }
        self.immutable.entries.insert(symbol, known);
        known
    }

    fn safe_uses(&mut self, symbol: SymbolId, depth: usize) -> bool {
        if depth >= 16 || self.exported.contains(&symbol.index()) {
            return false;
        }
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return false;
            }
            if !self.fresh_context(symbol, reference.node_id()) {
                return false;
            }
            let span = semantic.nodes().kind(reference.node_id()).span();
            let mut allowed = false;
            for node in semantic.nodes().ancestors(reference.node_id()).take(16) {
                if !self.take() {
                    return false;
                }
                match node.kind() {
                    AstKind::CallExpression(call) => {
                        allowed =
                            call.span.start == self.call_offset
                                && call.arguments.iter().take(32).any(|a| {
                                    a.span().start <= span.start && a.span().end >= span.end
                                });
                        break;
                    }
                    AstKind::NewExpression(call) => {
                        allowed =
                            call.span.start == self.call_offset
                                && call.arguments.iter().take(32).any(|a| {
                                    a.span().start <= span.start && a.span().end >= span.end
                                });
                        break;
                    }
                    AstKind::VariableDeclarator(declaration) => {
                        // Only direct immutable aliases preserve an aggregate's identity.
                        let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
                            break;
                        };
                        let Some(alias) = binding.symbol_id.get() else {
                            break;
                        };
                        let Some(Expression::Identifier(initializer)) = declaration
                            .init
                            .as_ref()
                            .map(Expression::get_inner_expression)
                        else {
                            break;
                        };
                        let Some(id) = initializer.reference_id.get() else {
                            break;
                        };
                        allowed = semantic.scoping().symbol_flags(alias).is_const_variable()
                            && semantic.scoping().get_reference(id).symbol_id() == Some(symbol)
                            && self.safe_uses(alias, depth + 1);
                        break;
                    }
                    AstKind::Function(_)
                    | AstKind::ArrowFunctionExpression(_)
                    | AstKind::AssignmentExpression(_)
                    | AstKind::UpdateExpression(_) => break,
                    _ => {}
                }
            }
            if !allowed {
                return false;
            }
        }
        true
    }
}

#[path = "context.rs"]
mod context;

#[cfg(test)]
#[path = "alias_tests.rs"]
mod tests;
