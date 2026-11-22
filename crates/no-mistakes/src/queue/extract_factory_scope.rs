use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    BlockStatement, CatchClause, Class, Expression, ForInStatement, ForOfStatement, ForStatement,
    ForStatementInit, ForStatementLeft, FunctionBody, Statement, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};

pub(crate) fn visit_block<'a>(visitor: &mut QueueVisitor<'a>, block: &BlockStatement<'a>) {
    let numbers = visitor.facts.const_numbers.clone();
    crate::queue::number_scope::prepare_statements(visitor, &block.body);
    let frames = visitor.job_frames.clone();
    mask_statements(visitor, &block.body);
    walk::walk_block_statement(visitor, block);
    visitor.job_frames = frames;
    visitor.facts.const_numbers = numbers;
}

pub(crate) fn visit_body<'a>(visitor: &mut QueueVisitor<'a>, body: &FunctionBody<'a>) {
    crate::queue::number_scope::prepare_statements(visitor, &body.statements);
    mask_statements(visitor, &body.statements);
    walk::walk_function_body(visitor, body);
}

pub(crate) fn visit_catch<'a>(visitor: &mut QueueVisitor<'a>, catch: &CatchClause<'a>) {
    let numbers = visitor.facts.const_numbers.clone();
    let frames = visitor.job_frames.clone();
    if let Some(param) = &catch.param {
        super::dedup::mask_pattern(visitor, &param.pattern);
        crate::queue::number_scope::hide_pattern(visitor, &param.pattern);
    }
    walk::walk_catch_clause(visitor, catch);
    visitor.job_frames = frames;
    visitor.facts.const_numbers = numbers;
}

pub(crate) fn visit_for<'a>(visitor: &mut QueueVisitor<'a>, stmt: &ForStatement<'a>) {
    let numbers = visitor.facts.const_numbers.clone();
    if let Some(ForStatementInit::VariableDeclaration(decl)) = &stmt.init {
        hide_declaration(visitor, decl);
    }
    walk::walk_for_statement(visitor, stmt);
    visitor.facts.const_numbers = numbers;
}

pub(crate) fn visit_for_in<'a>(visitor: &mut QueueVisitor<'a>, stmt: &ForInStatement<'a>) {
    visit_for_each(visitor, &stmt.left, &stmt.right, &stmt.body);
}

pub(crate) fn visit_for_of<'a>(visitor: &mut QueueVisitor<'a>, stmt: &ForOfStatement<'a>) {
    visit_for_each(visitor, &stmt.left, &stmt.right, &stmt.body);
}

fn visit_for_each<'a>(
    visitor: &mut QueueVisitor<'a>,
    left: &ForStatementLeft<'a>,
    right: &Expression<'a>,
    body: &Statement<'a>,
) {
    let numbers = visitor.facts.const_numbers.clone();
    // The for-of/for-in binding is in scope and uninitialized for the right-hand side.
    if let ForStatementLeft::VariableDeclaration(decl) = left {
        hide_declaration(visitor, decl);
    }
    visitor.visit_expression(right);
    walk::walk_for_statement_left(visitor, left);
    visitor.visit_statement(body);
    visitor.facts.const_numbers = numbers;
}

fn hide_declaration(visitor: &mut QueueVisitor<'_>, decl: &VariableDeclaration<'_>) {
    for declarator in &decl.declarations {
        crate::queue::number_scope::hide_pattern(visitor, &declarator.id);
    }
}

pub(crate) fn visit_class<'a>(visitor: &mut QueueVisitor<'a>, class: &Class<'a>) {
    let numbers = visitor.facts.const_numbers.clone();
    let frames = visitor.job_frames.clone();
    if let Some(id) = &class.id {
        mask_name(visitor, id.name.as_str());
        visitor.facts.const_numbers.remove(id.name.as_str());
    }
    walk::walk_class(visitor, class);
    visitor.job_frames = frames;
    visitor.facts.const_numbers = numbers;
}

pub(super) fn mask_name(visitor: &mut QueueVisitor<'_>, name: &str) {
    for frame in &mut visitor.job_frames {
        if frame.object.as_deref() == Some(name) {
            frame.object = None;
        }
        frame.id_idents.retain(|ident| ident != name);
        frame.name_idents.retain(|ident| ident != name);
    }
}

fn mask_statements(visitor: &mut QueueVisitor<'_>, statements: &[Statement<'_>]) {
    // Lexical declarations shadow the whole block, including earlier statements.
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    super::dedup::mask_pattern(visitor, &declarator.id);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    mask_name(visitor, id.name.as_str());
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    mask_name(visitor, id.name.as_str());
                }
            }
            _ => {}
        }
    }
}
