use super::super::{expressions::expression, trust, Expr};
use crate::fx::FxHashSet;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(super) struct Calls<'s> {
    pub source: &'s str,
    pub covered: FxHashSet<u32>,
    pub values: Vec<Expr>,
    pub scopes: Vec<FxHashSet<String>>,
}
impl Calls<'_> {
    fn params(&mut self, params: &FormalParameters<'_>) {
        let mut names = params
            .items
            .iter()
            .flat_map(|param| trust::bound_names(&param.pattern))
            .collect::<FxHashSet<_>>();
        if let Some(rest) = &params.rest {
            names.extend(trust::bound_names(&rest.rest.argument));
        }
        self.scopes.push(names);
    }
    fn body(&mut self, body: &[Statement<'_>]) {
        self.scopes
            .push(trust::declared_names(body).into_iter().collect());
    }
    fn binding(&mut self, pattern: Option<&BindingPattern<'_>>) {
        self.scopes
            .push(pattern.into_iter().flat_map(trust::bound_names).collect());
    }
}
impl<'a> Visit<'a> for Calls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if !self.covered.contains(&call.span.start) {
            let mut callee = expression(&call.callee, self.source);
            // An unmodeled local binding must never resolve as a same-name
            // module helper. Only module/import identities remain provable.
            if matches!(&callee, Expr::Name(name) if self.scopes.iter().any(|scope| scope.contains(name)))
            {
                callee = Expr::Unknown;
            }
            self.values.push(Expr::Call {
                callee: Box::new(callee),
                args: call
                    .arguments
                    .iter()
                    .map(|arg| {
                        let expr = arg
                            .as_expression()
                            .map_or(Expr::Unknown, |arg| expression(arg, self.source));
                        match &expr {
                            Expr::Name(name)
                                if !self.scopes.iter().any(|scope| scope.contains(name)) =>
                            {
                                Expr::OpaqueCallback(Box::new(expr))
                            }
                            Expr::Function(_) => Expr::OpaqueCallback(Box::new(expr)),
                            _ => Expr::Unknown,
                        }
                    })
                    .collect(),
                start: call.span.start,
            });
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        self.params(&function.params);
        if function.r#type == FunctionType::FunctionExpression {
            if let Some(id) = &function.id {
                self.scopes.last_mut().unwrap().insert(id.name.to_string());
            }
        }
        walk::walk_function(self, function, flags);
        self.scopes.pop();
    }
    fn visit_arrow_function_expression(&mut self, function: &ArrowFunctionExpression<'a>) {
        self.params(&function.params);
        walk::walk_arrow_function_expression(self, function);
        self.scopes.pop();
    }
    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.body(&body.statements);
        walk::walk_function_body(self, body);
        self.scopes.pop();
    }
    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.body(&block.body);
        walk::walk_block_statement(self, block);
        self.scopes.pop();
    }
    fn visit_catch_clause(&mut self, value: &CatchClause<'a>) {
        self.binding(value.param.as_ref().map(|param| &param.pattern));
        walk::walk_catch_clause(self, value);
        self.scopes.pop();
    }
    fn visit_switch_statement(&mut self, value: &SwitchStatement<'a>) {
        self.visit_expression(&value.discriminant);
        let names = value
            .cases
            .iter()
            .flat_map(|case| trust::declared_names(&case.consequent))
            .collect();
        self.scopes.push(names);
        for case in &value.cases {
            self.visit_switch_case(case);
        }
        self.scopes.pop();
    }
    fn visit_ts_module_block(&mut self, value: &TSModuleBlock<'a>) {
        self.body(&value.body);
        walk::walk_ts_module_block(self, value);
        self.scopes.pop();
    }
    fn visit_static_block(&mut self, value: &StaticBlock<'a>) {
        self.body(&value.body);
        walk::walk_static_block(self, value);
        self.scopes.pop();
    }
    fn visit_for_statement(&mut self, value: &ForStatement<'a>) {
        let names = match &value.init {
            Some(ForStatementInit::VariableDeclaration(value)) => value
                .declarations
                .iter()
                .flat_map(|value| trust::bound_names(&value.id))
                .collect(),
            _ => FxHashSet::default(),
        };
        self.scopes.push(names);
        walk::walk_for_statement(self, value);
        self.scopes.pop();
    }
    fn visit_for_in_statement(&mut self, value: &ForInStatement<'a>) {
        let names = match &value.left {
            ForStatementLeft::VariableDeclaration(value) => value
                .declarations
                .iter()
                .flat_map(|value| trust::bound_names(&value.id))
                .collect(),
            _ => FxHashSet::default(),
        };
        self.scopes.push(names);
        walk::walk_for_in_statement(self, value);
        self.scopes.pop();
    }
    fn visit_for_of_statement(&mut self, value: &ForOfStatement<'a>) {
        let names = match &value.left {
            ForStatementLeft::VariableDeclaration(value) => value
                .declarations
                .iter()
                .flat_map(|value| trust::bound_names(&value.id))
                .collect(),
            _ => FxHashSet::default(),
        };
        self.scopes.push(names);
        walk::walk_for_of_statement(self, value);
        self.scopes.pop();
    }
}
