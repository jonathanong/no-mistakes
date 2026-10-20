use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{BlockStatement, CatchClause, Class, FunctionBody, Statement};
use oxc_ast_visit::{walk, Visit};

pub(crate) fn visit_block<'a>(visitor: &mut QueueVisitor<'a>, block: &BlockStatement<'a>) {
    let frames = visitor.job_frames.clone();
    mask_statements(visitor, &block.body);
    walk::walk_block_statement(visitor, block);
    visitor.job_frames = frames;
}

pub(crate) fn visit_body<'a>(visitor: &mut QueueVisitor<'a>, body: &FunctionBody<'a>) {
    mask_statements(visitor, &body.statements);
    walk::walk_function_body(visitor, body);
}

pub(crate) fn visit_catch<'a>(visitor: &mut QueueVisitor<'a>, catch: &CatchClause<'a>) {
    let frames = visitor.job_frames.clone();
    if let Some(param) = &catch.param {
        super::dedup::mask_pattern(visitor, &param.pattern);
    }
    walk::walk_catch_clause(visitor, catch);
    visitor.job_frames = frames;
}

pub(crate) fn visit_class<'a>(visitor: &mut QueueVisitor<'a>, class: &Class<'a>) {
    let frames = visitor.job_frames.clone();
    if let Some(id) = &class.id {
        mask_name(visitor, id.name.as_str());
    }
    walk::walk_class(visitor, class);
    visitor.job_frames = frames;
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
