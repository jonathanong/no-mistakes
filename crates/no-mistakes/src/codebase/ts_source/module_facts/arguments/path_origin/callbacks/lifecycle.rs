use super::*;
use oxc_semantic::SymbolId;

impl<'a> Projector<'a, '_> {
    pub(super) fn finally_cleans(&mut self, call: NodeId, root: SymbolId, helper: u32) -> bool {
        let semantic = self.semantic;
        let nodes = semantic.nodes();
        let offset = nodes.kind(call).span().start;
        for ancestor in nodes.ancestors(call) {
            if ancestor.kind().span().start == helper {
                break;
            }
            let AstKind::TryStatement(tried) = ancestor.kind() else {
                continue;
            };
            if offset <= tried.block.span.start || offset >= tried.block.span.end {
                continue;
            }
            let Some(finalizer) = &tried.finalizer else {
                continue;
            };
            let Some(Statement::ExpressionStatement(statement)) = finalizer.body.first() else {
                continue;
            };
            let (expression, awaited) = match statement.expression.get_inner_expression() {
                Expression::AwaitExpression(inner) => (&inner.argument, true),
                other => (other, false),
            };
            let Expression::CallExpression(cleanup) = expression.get_inner_expression() else {
                continue;
            };
            if self.cleanup_binding(cleanup, awaited) && self.cleanup_argument(cleanup, root) {
                return true;
            }
        }
        false
    }

    pub(super) fn cleanup_call(
        &mut self,
        call: &CallExpression<'a>,
        root: SymbolId,
        node: NodeId,
    ) -> bool {
        let awaited = matches!(
            self.semantic.nodes().parent_kind(node),
            AstKind::AwaitExpression(_)
        );
        self.cleanup_binding(call, awaited) && self.cleanup_argument(call, root)
    }

    fn cleanup_argument(&self, call: &CallExpression<'_>, root: SymbolId) -> bool {
        let Some(Expression::Identifier(id)) = call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression)
        else {
            return false;
        };
        if self.binding_symbol(id) != Some(root) {
            return false;
        }
        let Some(Expression::ObjectExpression(options)) = call
            .arguments
            .get(1)
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression)
        else {
            return false;
        };
        let mut recursive = false;
        for property in &options.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            if property.computed || property.method || property.kind != PropertyKind::Init {
                return false;
            }
            if crate::ast::property_key_name(&property.key).as_deref() == Some("recursive") {
                recursive = matches!(property.value.get_inner_expression(), Expression::BooleanLiteral(value) if value.value);
            }
        }
        recursive
    }

    fn cleanup_binding(&mut self, call: &CallExpression<'a>, awaited: bool) -> bool {
        let Expression::Identifier(id) = call.callee.get_inner_expression() else {
            return false;
        };
        let Some(symbol) = self.binding_symbol(id) else {
            return false;
        };
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.is_write() {
                return false;
            }
        }
        let declaration = semantic.scoping().symbol_declaration(symbol);
        let AstKind::ImportSpecifier(binding) = semantic.nodes().kind(declaration) else {
            return false;
        };
        if binding.import_kind.is_type() {
            return false;
        }
        let name = crate::codebase::ts_source::module_facts::modules::name(&binding.imported);
        let AstKind::ImportDeclaration(import) = semantic.nodes().parent_kind(declaration) else {
            return false;
        };
        if import.import_kind.is_type() {
            return false;
        }
        let source = crate::ast::js_literal(&import.source);
        match name.as_str() {
            "rmSync" => matches!(source.as_ref(), "fs" | "node:fs"),
            "rm" => awaited && matches!(source.as_ref(), "fs/promises" | "node:fs/promises"),
            _ => false,
        }
    }
}
