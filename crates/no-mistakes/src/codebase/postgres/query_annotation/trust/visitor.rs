use super::assigned::Assigned;
use super::bindings::{bound, function_scope, lexical};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;

pub(super) fn collect(program: &Program<'_>) -> HashSet<String> {
    let mut visitor = Writes {
        scopes: vec![function_scope(&program.body)],
        names: HashSet::new(),
        module_functions: super::bindings::module_functions(&program.body),
    };
    visitor.visit_program(program);
    visitor.names
}

struct Writes {
    scopes: Vec<HashSet<String>>,
    names: HashSet<String>,
    module_functions: HashSet<String>,
}
impl Writes {
    fn write(&mut self, name: &str) {
        // An unbound assignment can change the conventional global SQL tag.
        let owner = self.scopes.iter().rposition(|scope| scope.contains(name));
        if owner.is_none_or(|owner| owner == 0) {
            self.names.insert(name.to_string());
        }
    }
    fn parameters(&mut self, params: &FormalParameters<'_>, mut names: HashSet<String>) {
        for param in &params.items {
            bound(&param.pattern, &mut names);
        }
        if let Some(rest) = &params.rest {
            bound(&rest.rest.argument, &mut names);
        }
        self.scopes.push(names);
    }
    fn loop_scope(&mut self, declaration: Option<&VariableDeclaration<'_>>) {
        let mut names = HashSet::new();
        if let Some(value) = declaration.filter(|value| value.kind != VariableDeclarationKind::Var)
        {
            for value in &value.declarations {
                bound(&value.id, &mut names);
            }
        }
        self.scopes.push(names);
    }
}

impl<'a> Visit<'a> for Writes {
    fn visit_function(&mut self, value: &Function<'a>, flags: ScopeFlags) {
        let mut names = HashSet::new();
        if value.r#type == FunctionType::FunctionExpression {
            if let Some(id) = &value.id {
                names.insert(id.name.to_string());
            }
        }
        self.parameters(&value.params, names);
        walk::walk_function(self, value, flags);
        self.scopes.pop();
    }
    fn visit_arrow_function_expression(&mut self, value: &ArrowFunctionExpression<'a>) {
        self.parameters(&value.params, HashSet::new());
        walk::walk_arrow_function_expression(self, value);
        self.scopes.pop();
    }
    fn visit_function_body(&mut self, value: &FunctionBody<'a>) {
        // Parameter initializers run before body declarations enter scope.
        self.scopes.push(function_scope(&value.statements));
        walk::walk_function_body(self, value);
        self.scopes.pop();
    }
    fn visit_block_statement(&mut self, value: &BlockStatement<'a>) {
        self.scopes.push(lexical(&value.body));
        walk::walk_block_statement(self, value);
        self.scopes.pop();
    }
    fn visit_catch_clause(&mut self, value: &CatchClause<'a>) {
        let mut names = HashSet::new();
        if let Some(param) = &value.param {
            bound(&param.pattern, &mut names);
        }
        self.scopes.push(names);
        walk::walk_catch_clause(self, value);
        self.scopes.pop();
    }
    fn visit_class(&mut self, value: &Class<'a>) {
        let names = value
            .id
            .as_ref()
            .map(|id| HashSet::from([id.name.to_string()]))
            .unwrap_or_default();
        self.scopes.push(names);
        walk::walk_class(self, value);
        self.scopes.pop();
    }
    fn visit_static_block(&mut self, value: &StaticBlock<'a>) {
        self.scopes.push(function_scope(&value.body));
        walk::walk_static_block(self, value);
        self.scopes.pop();
    }
    fn visit_ts_module_block(&mut self, value: &TSModuleBlock<'a>) {
        self.scopes.push(function_scope(&value.body));
        walk::walk_ts_module_block(self, value);
        self.scopes.pop();
    }
    fn visit_switch_statement(&mut self, value: &SwitchStatement<'a>) {
        self.visit_expression(&value.discriminant);
        let names = value
            .cases
            .iter()
            .flat_map(|case| lexical(&case.consequent))
            .collect();
        self.scopes.push(names);
        self.visit_switch_cases(&value.cases);
        self.scopes.pop();
    }
    fn visit_for_statement(&mut self, value: &ForStatement<'a>) {
        self.loop_scope(value.init.as_ref().and_then(|init| match init {
            ForStatementInit::VariableDeclaration(value) => Some(value.as_ref()),
            _ => None,
        }));
        walk::walk_for_statement(self, value);
        self.scopes.pop();
    }
    fn visit_for_in_statement(&mut self, value: &ForInStatement<'a>) {
        self.loop_scope(match &value.left {
            ForStatementLeft::VariableDeclaration(value) => Some(value.as_ref()),
            _ => None,
        });
        walk::walk_for_in_statement(self, value);
        self.scopes.pop();
    }
    fn visit_for_of_statement(&mut self, value: &ForOfStatement<'a>) {
        self.loop_scope(match &value.left {
            ForStatementLeft::VariableDeclaration(value) => Some(value.as_ref()),
            _ => None,
        });
        walk::walk_for_of_statement(self, value);
        self.scopes.pop();
    }
    fn visit_for_statement_left(&mut self, value: &ForStatementLeft<'a>) {
        if let ForStatementLeft::VariableDeclaration(declaration) = value {
            if declaration.kind == VariableDeclarationKind::Var {
                let mut names = HashSet::new();
                for value in &declaration.declarations {
                    bound(&value.id, &mut names);
                }
                for name in names {
                    self.write(&name);
                }
            }
        }
        walk::walk_for_statement_left(self, value);
    }
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind == VariableDeclarationKind::Var {
            let mut names = HashSet::new();
            for value in &value.declarations {
                if value.init.is_some() {
                    bound(&value.id, &mut names);
                }
            }
            for name in names {
                if self.module_functions.contains(&name) {
                    self.write(&name);
                }
            }
        }
        walk::walk_variable_declaration(self, value);
    }
    fn visit_assignment_target(&mut self, value: &AssignmentTarget<'a>) {
        let mut names = Assigned::default();
        names.visit_assignment_target(value);
        for name in names.0 {
            self.write(&name);
        }
        walk::walk_assignment_target(self, value);
    }
    fn visit_simple_assignment_target(&mut self, value: &SimpleAssignmentTarget<'a>) {
        if super::raw::target(value) {
            self.write("String");
        }
        walk::walk_simple_assignment_target(self, value);
    }
    fn visit_unary_expression(&mut self, value: &UnaryExpression<'a>) {
        if value.operator == oxc_ast::ast::UnaryOperator::Delete
            && super::raw::expression(&value.argument)
        {
            self.write("String");
        }
        walk::walk_unary_expression(self, value);
    }
    fn visit_update_expression(&mut self, value: &UpdateExpression<'a>) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &value.argument {
            self.write(id.name.as_str());
        }
        walk::walk_update_expression(self, value);
    }
}
