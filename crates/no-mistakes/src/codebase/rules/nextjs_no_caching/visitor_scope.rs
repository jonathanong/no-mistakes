use super::const_values::{self, record_const};
use super::patterns::single_binding_name;
use super::visitor::NextjsCachingVisitor;
use oxc_ast::ast::{VariableDeclaration, VariableDeclarationKind};

impl<'a> NextjsCachingVisitor<'a> {
    pub(crate) fn push_const_scope(&mut self) {
        const_values::push_scope(&mut self.const_scopes);
    }

    pub(crate) fn pop_const_scope(&mut self) {
        const_values::pop_scope(&mut self.const_scopes);
    }

    pub(crate) fn record_const_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind != VariableDeclarationKind::Const {
            return;
        }
        for declarator in &declaration.declarations {
            if let (Some(name), Some(init)) = (
                single_binding_name(&declarator.id),
                declarator.init.as_ref(),
            ) {
                record_const(&mut self.const_scopes, &name, init);
            }
        }
    }
}
