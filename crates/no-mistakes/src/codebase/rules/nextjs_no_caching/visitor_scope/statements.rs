use super::super::visitor::NextjsCachingVisitor;
use oxc_ast::ast::{
    CatchClause, Class, ForStatement, ForStatementInit, ForStatementLeft, StaticBlock,
    SwitchStatement,
};

impl<'a> NextjsCachingVisitor<'a> {
    pub(crate) fn open_catch_scope(&mut self, catch: &CatchClause<'a>) {
        self.push_const_scope();
        if let Some(param) = &catch.param {
            self.record_pattern_unknown(&param.pattern);
        }
    }

    pub(crate) fn open_class_scope(&mut self, class: &Class<'a>) {
        self.push_const_scope();
        self.record_name(class.id.as_ref().map(|id| id.name.as_str()));
    }

    pub(crate) fn open_static_scope(&mut self, block: &StaticBlock<'a>) {
        self.push_const_scope();
        self.precollect_hoisted_vars(&block.body);
        self.precollect_statements(&block.body);
    }

    pub(crate) fn open_for_scope(&mut self, statement: &ForStatement<'a>) {
        self.open_loop_scope(match &statement.init {
            Some(ForStatementInit::VariableDeclaration(declaration)) => Some(declaration),
            _ => None,
        });
    }

    pub(crate) fn open_for_left_scope(&mut self, left: &ForStatementLeft<'a>) {
        self.open_loop_scope(match left {
            ForStatementLeft::VariableDeclaration(declaration) => Some(declaration),
            _ => None,
        });
    }

    pub(crate) fn open_switch_scope(&mut self, statement: &SwitchStatement<'a>) {
        self.push_const_scope();
        self.precollect_statements(statement.cases.iter().flat_map(|case| &case.consequent));
    }
}
