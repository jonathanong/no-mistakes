use oxc_ast::ast::{
    ArrowFunctionExpression, CatchClause, ConditionalExpression, DoWhileStatement, ForInStatement,
    ForOfStatement, ForStatement, Function, IfStatement, LogicalExpression, MethodDefinition,
    Program, SwitchCase, WhileStatement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;

#[derive(Debug, Clone)]
pub struct FunctionSpan {
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub ast_complexity: u64,
}

struct FunctionCollector<'s> {
    source: &'s str,
    stack: Vec<(String, u32, u32, u64)>,
    finished: Vec<FunctionSpan>,
}

impl FunctionCollector<'_> {
    fn bump(&mut self) {
        if let Some(top) = self.stack.last_mut() {
            top.3 += 1;
        }
    }

    fn enter(&mut self, name: String, start: u32, end: u32) {
        self.stack.push((name, start, end, 1));
    }

    fn exit(&mut self) {
        if let Some((name, start, end, ast_complexity)) = self.stack.pop() {
            self.finished.push(FunctionSpan {
                name,
                start,
                end,
                ast_complexity,
            });
        }
    }
}

impl<'a> Visit<'a> for FunctionCollector<'_> {
    fn visit_function(&mut self, it: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if it.body.is_some() {
            let name = it
                .id
                .as_ref()
                .map(|id| id.name.as_str().to_string())
                .unwrap_or_else(|| format!("anon@{}", it.span.start));
            self.enter(name, it.span.start, it.span.end);
            walk::walk_function(self, it, flags);
            self.exit();
        } else {
            walk::walk_function(self, it, flags);
        }
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        self.enter(
            format!("arrow@{}", it.span.start),
            it.span.start,
            it.span.end,
        );
        walk::walk_arrow_function_expression(self, it);
        self.exit();
    }

    fn visit_method_definition(&mut self, it: &MethodDefinition<'a>) {
        let ks = it.key.span();
        let name = self
            .source
            .get(ks.start as usize..ks.end as usize)
            .unwrap_or("method");
        if let Some(body) = it.value.body.as_ref() {
            self.enter(name.to_string(), it.span.start, it.span.end);
            walk::walk_function_body(self, body);
            self.exit();
        } else {
            walk::walk_method_definition(self, it);
        }
    }

    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        self.bump();
        walk::walk_if_statement(self, it);
    }

    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        self.bump();
        walk::walk_for_statement(self, it);
    }

    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        self.bump();
        walk::walk_for_in_statement(self, it);
    }

    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        self.bump();
        walk::walk_for_of_statement(self, it);
    }

    fn visit_while_statement(&mut self, it: &WhileStatement<'a>) {
        self.bump();
        walk::walk_while_statement(self, it);
    }

    fn visit_do_while_statement(&mut self, it: &DoWhileStatement<'a>) {
        self.bump();
        walk::walk_do_while_statement(self, it);
    }

    fn visit_switch_case(&mut self, it: &SwitchCase<'a>) {
        if it.test.is_some() {
            self.bump();
        }
        walk::walk_switch_case(self, it);
    }

    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        self.bump();
        walk::walk_catch_clause(self, it);
    }

    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        self.bump();
        walk::walk_conditional_expression(self, it);
    }

    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.bump();
        walk::walk_logical_expression(self, it);
    }
}

pub fn collect_functions<'a>(program: &Program<'a>, source: &str) -> Vec<FunctionSpan> {
    let mut collector = FunctionCollector {
        source,
        stack: Vec::new(),
        finished: Vec::new(),
    };
    collector.visit_program(program);
    collector.finished.sort_by_key(|f| (f.start, f.end));
    collector.finished
}
