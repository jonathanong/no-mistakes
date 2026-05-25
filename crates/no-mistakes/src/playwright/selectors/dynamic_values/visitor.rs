use super::collect::{
    binding_identifier_name, call_identifier_name, collect_assignments_from_stmt,
    collect_object_string_values, collect_string_leaves, extract_computed_member_object_name,
};
use super::DynamicIdentifierValues;
use oxc_ast::ast::{Expression, Statement};
use oxc_ast_visit::Visit;
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;

pub(super) struct DynamicValuesVisitor {
    pub(super) collected: Vec<DynamicIdentifierValues>,
    scope_stack: Vec<Span>,
}

impl DynamicValuesVisitor {
    pub(super) fn new() -> Self {
        Self {
            collected: Vec::new(),
            scope_stack: Vec::new(),
        }
    }

    fn current_scope(&self) -> Option<Span> {
        self.scope_stack.last().copied()
    }

    pub(super) fn collect_from_statements(&mut self, statements: &[Statement<'_>]) {
        let Some(scope) = self.current_scope() else {
            return;
        };

        let mut object_map: Vec<(String, Vec<String>)> = Vec::new();

        for stmt in statements {
            if let Statement::VariableDeclaration(var_decl) = stmt {
                for declarator in &var_decl.declarations {
                    self.collect_from_declarator(declarator, scope, &object_map);
                    if let (Some(name), Some(init)) = (
                        binding_identifier_name(&declarator.id),
                        declarator.init.as_ref(),
                    ) {
                        let obj_vals = collect_object_string_values(init);
                        if !obj_vals.is_empty() {
                            object_map.push((name, obj_vals));
                        }
                    }
                }
            }
        }

        for stmt in statements {
            match stmt {
                Statement::IfStatement(if_stmt) => {
                    self.collect_from_if_statement(if_stmt, scope);
                }
                Statement::SwitchStatement(switch_stmt) => {
                    self.collect_from_switch(&switch_stmt.cases, scope);
                }
                _ => {}
            }
        }
    }

    fn collect_from_declarator(
        &mut self,
        declarator: &oxc_ast::ast::VariableDeclarator<'_>,
        scope: Span,
        object_map: &[(String, Vec<String>)],
    ) {
        let Some(name) = binding_identifier_name(&declarator.id) else {
            return;
        };
        let Some(init) = declarator.init.as_ref() else {
            return;
        };

        let leaves = collect_string_leaves(init);
        if !leaves.is_empty() {
            self.push(name, leaves, scope);
            return;
        }

        let computed_obj_name = extract_computed_member_object_name(init);
        if let Some(obj_name) = computed_obj_name {
            let values: Vec<String> = object_map
                .iter()
                .filter(|(n, _)| n == obj_name)
                .flat_map(|(_, v)| v.clone())
                .collect();
            if !values.is_empty() {
                self.push(name, values, scope);
                return;
            }
            self.push(name, vec!["__obj__".to_string() + obj_name], scope);
            return;
        }

        if let Expression::CallExpression(call) = init {
            if let Some(fn_name) = call_identifier_name(&call.callee) {
                self.push(name, vec!["__call__".to_string() + &fn_name], scope);
            }
        }
    }

    fn collect_from_if_statement(&mut self, if_stmt: &oxc_ast::ast::IfStatement<'_>, scope: Span) {
        let mut all_names: Vec<String> = Vec::new();
        let mut by_name: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        collect_assignments_from_stmt(&if_stmt.consequent, &mut |name, value| {
            if !all_names.iter().any(|s| s == name) {
                all_names.push(name.to_string());
            }
            by_name
                .entry(name.to_string())
                .or_default()
                .push(value.to_string());
        });

        if let Some(alt) = &if_stmt.alternate {
            collect_assignments_from_stmt(alt, &mut |name, value| {
                if !all_names.iter().any(|s| s == name) {
                    all_names.push(name.to_string());
                }
                by_name
                    .entry(name.to_string())
                    .or_default()
                    .push(value.to_string());
            });
        }

        for name in all_names {
            if let Some(values) = by_name.remove(&name) {
                self.push(name, values, scope);
            }
        }
    }

    fn collect_from_switch(&mut self, cases: &[oxc_ast::ast::SwitchCase<'_>], scope: Span) {
        let mut all_names: Vec<String> = Vec::new();
        let mut by_name: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for case in cases {
            for stmt in &case.consequent {
                collect_assignments_from_stmt(stmt, &mut |name, value| {
                    if !all_names.iter().any(|s| s == name) {
                        all_names.push(name.to_string());
                    }
                    by_name
                        .entry(name.to_string())
                        .or_default()
                        .push(value.to_string());
                });
            }
        }

        for name in all_names {
            if let Some(values) = by_name.remove(&name) {
                self.push(name, values, scope);
            }
        }
    }

    fn push(&mut self, name: String, values: Vec<String>, scope: Span) {
        self.collected.push(DynamicIdentifierValues {
            name,
            values,
            scope,
        });
    }
}

impl<'a> Visit<'a> for DynamicValuesVisitor {
    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        if let Some(body) = &function.body {
            self.scope_stack.push(body.span());
            self.collect_from_statements(&body.statements);
            oxc_ast_visit::walk::walk_function(self, function, flags);
            self.scope_stack.pop();
        }
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        let scope = arrow.body.span();
        self.scope_stack.push(scope);
        self.collect_from_statements(&arrow.body.statements);
        oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
        self.scope_stack.pop();
    }
}
