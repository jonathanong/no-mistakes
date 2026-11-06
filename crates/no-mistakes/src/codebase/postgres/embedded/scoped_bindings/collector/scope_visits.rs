macro_rules! scope_visits {
    () => {
        fn visit_program(&mut self, program: &Program<'a>) {
            self.with_scope(program.span, |this| walk::walk_program(this, program));
        }

        fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
            self.with_scope(block.span, |this| walk::walk_block_statement(this, block));
        }

        fn visit_ts_module_block(&mut self, block: &oxc_ast::ast::TSModuleBlock<'a>) {
            self.with_scope(block.span, |this| walk::walk_ts_module_block(this, block));
        }

        fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
            self.with_scope(body.span, |this| walk::walk_function_body(this, body));
        }

        fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
            self.found.undeclared_this_exclusions.push(block.span);
            self.with_scope(block.span, |this| walk::walk_static_block(this, block));
        }

        fn visit_switch_statement(&mut self, switch: &SwitchStatement<'a>) {
            self.with_scope(switch.span, |this| {
                walk::walk_switch_statement(this, switch)
            });
        }

        fn visit_catch_clause(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
            self.with_scope(clause.span, |this| walk::walk_catch_clause(this, clause));
        }

        fn visit_for_in_statement(&mut self, statement: &oxc_ast::ast::ForInStatement<'a>) {
            self.with_scope(statement.span, |this| {
                walk::walk_for_in_statement(this, statement)
            });
        }

        fn visit_for_of_statement(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
            self.with_scope(statement.span, |this| {
                walk::walk_for_of_statement(this, statement)
            });
        }

        fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
            self.with_scope(statement.span, |this| {
                walk::walk_for_statement(this, statement)
            });
        }
    };
}
