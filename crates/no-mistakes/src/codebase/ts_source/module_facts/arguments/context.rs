use super::*;
use oxc_syntax::node::NodeId;

impl Projector<'_, '_> {
    pub(super) fn fresh_context(&mut self, symbol: SymbolId, reference: NodeId) -> bool {
        let semantic = self.semantic;
        let declaration = semantic
            .nodes()
            .kind(semantic.scoping().symbol_declaration(symbol))
            .span();
        for node in semantic.nodes().ancestors(reference) {
            if !self.take() {
                return false;
            }
            match node.kind() {
                // A callee may mutate an aggregate, so a later execution cannot
                // reuse a carrier allocated outside the current callable.
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    let span = node.kind().span();
                    if declaration.start < span.start || declaration.end > span.end {
                        return false;
                    }
                }
                AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::PropertyDefinition(_) => return false,
                _ => {}
            }
        }
        true
    }
}
