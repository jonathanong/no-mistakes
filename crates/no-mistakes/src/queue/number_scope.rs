use crate::queue::extract_model::{EnqueuePriority, FileFacts};
use crate::queue::extract_priority::const_nonnegative_integer;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, FormalParameters, Function, Program, Statement,
    VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::Visit;

pub(super) fn seed_program(visitor: &mut QueueVisitor<'_>, program: &Program<'_>) {
    prepare_statements(visitor, &program.body);
}

pub(super) fn prepare_statements(visitor: &mut QueueVisitor<'_>, statements: &[Statement<'_>]) {
    hide_statements(visitor, statements);
    let mut pending = direct_const_inits(statements);
    let mut changed = true;
    while changed {
        changed = false;
        let mut later = Vec::new();
        for (name, init) in pending {
            if let Some(value) = const_nonnegative_integer(init, &visitor.facts.const_numbers) {
                visitor.facts.const_numbers.insert(name, value);
                changed = true;
            } else {
                later.push((name, init));
            }
        }
        pending = later;
    }
}

pub(super) fn hide_statements(visitor: &mut QueueVisitor<'_>, statements: &[Statement<'_>]) {
    hide(visitor, &direct_binding_names(statements));
}

pub(super) fn hide_params(visitor: &mut QueueVisitor<'_>, params: &FormalParameters<'_>) {
    let mut names = Vec::new();
    for param in &params.items {
        push_pattern(&mut names, &param.pattern);
    }
    if let Some(rest) = &params.rest {
        push_pattern(&mut names, &rest.rest.argument);
    }
    hide(visitor, &names);
}

pub(super) fn hide_pattern(visitor: &mut QueueVisitor<'_>, pattern: &BindingPattern<'_>) {
    let mut names = Vec::new();
    push_pattern(&mut names, pattern);
    hide(visitor, &names);
}

pub(super) fn hide_function_vars(visitor: &mut QueueVisitor<'_>, statements: &[Statement<'_>]) {
    let mut names = VarNames(Vec::new());
    for statement in statements {
        names.visit_statement(statement);
    }
    hide(visitor, &names.0);
}

pub(super) fn finish_opaque(facts: &mut FileFacts) {
    if facts.opaque_priority_bindings.is_empty() {
        return;
    }
    for producer in &mut facts.producers {
        if facts.opaque_priority_bindings.contains(&producer.binding) {
            producer.priority = EnqueuePriority::Dynamic;
        }
    }
}

fn hide(visitor: &mut QueueVisitor<'_>, names: &[String]) {
    for name in names {
        visitor.facts.const_numbers.remove(name);
    }
}

fn direct_binding_names(statements: &[Statement<'_>]) -> Vec<String> {
    let mut names = Vec::new();
    for statement in statements {
        collect_bindings(statement, &mut names);
    }
    names
}

fn collect_bindings(statement: &Statement<'_>, names: &mut Vec<String>) {
    match statement {
        Statement::VariableDeclaration(decl) => push_declaration(names, decl),
        Statement::FunctionDeclaration(func) => push_function_name(names, func),
        Statement::ClassDeclaration(class) => push_class_name(names, class),
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::VariableDeclaration(decl) => push_declaration(names, decl),
            Declaration::FunctionDeclaration(func) => push_function_name(names, func),
            Declaration::ClassDeclaration(class) => push_class_name(names, class),
            _ => {}
        },
        _ => {}
    }
}

fn push_declaration(names: &mut Vec<String>, decl: &VariableDeclaration<'_>) {
    for declarator in &decl.declarations {
        push_pattern(names, &declarator.id);
    }
}

fn push_function_name(names: &mut Vec<String>, func: &Function<'_>) {
    if let Some(id) = &func.id {
        names.push(id.name.to_string());
    }
}

fn push_class_name(names: &mut Vec<String>, class: &oxc_ast::ast::Class<'_>) {
    if let Some(id) = &class.id {
        names.push(id.name.to_string());
    }
}

fn direct_const_inits<'a>(statements: &'a [Statement<'a>]) -> Vec<(String, &'a Expression<'a>)> {
    let mut inits = Vec::new();
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(decl) => push_const_inits(&mut inits, decl),
            Statement::ExportDeclaration(export) => {
                if let Declaration::VariableDeclaration(decl) = &export.declaration {
                    push_const_inits(&mut inits, decl);
                }
            }
            _ => {}
        }
    }
    inits
}

fn push_const_inits<'a>(
    inits: &mut Vec<(String, &'a Expression<'a>)>,
    decl: &'a VariableDeclaration<'a>,
) {
    if decl.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &decl.declarations {
        if let Some(name) = crate::queue::extract_helpers::binding_name(&declarator.id) {
            if let Some(init) = &declarator.init {
                inits.push((name, init));
            }
        }
    }
}

fn push_pattern(names: &mut Vec<String>, pattern: &BindingPattern<'_>) {
    struct Names<'a>(&'a mut Vec<String>);
    impl Visit<'_> for Names<'_> {
        fn visit_binding_identifier(&mut self, id: &oxc_ast::ast::BindingIdentifier<'_>) {
            self.0.push(id.name.to_string());
        }

        fn visit_expression(&mut self, _expression: &Expression<'_>) {}
    }
    Names(names).visit_binding_pattern(pattern);
}

struct VarNames(Vec<String>);

impl Visit<'_> for VarNames {
    fn visit_function(&mut self, _func: &Function<'_>, _flags: oxc_syntax::scope::ScopeFlags) {}

    fn visit_arrow_function_expression(
        &mut self,
        _arrow: &oxc_ast::ast::ArrowFunctionExpression<'_>,
    ) {
    }

    fn visit_static_block(&mut self, _block: &oxc_ast::ast::StaticBlock<'_>) {}

    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'_>) {
        if decl.kind == VariableDeclarationKind::Var {
            push_declaration(&mut self.0, decl);
        }
    }
}
