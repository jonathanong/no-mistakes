//! Reference extraction from HCL expressions: walks expression trees and
//! classifies traversals into declarable Terraform addresses. Split from
//! `parse.rs` to keep each source file within the per-file line budget.

use hcl::expr::{Expression, Object, ObjectKey, Operation, Traversal, TraversalOperator};
use hcl::structure::{Block, Body, Structure};
use hcl::template::{Directive, Element, Template};
use std::path::Path;

use super::classify::traversal_to_addr;
use super::{TerraformRef, TfAddr};

/// Walk every attribute in a block body (recursing into nested blocks) and record
/// references attributed to the enclosing block address. `bound` carries the
/// iterator names of any enclosing `dynamic` block, which are locals.
pub(super) fn collect_body_refs(
    body: &Body,
    path: &Path,
    from_addr: &str,
    references: &mut Vec<TerraformRef>,
    bound: &[&str],
) {
    for structure in body.iter() {
        match structure {
            Structure::Attribute(attr) => {
                push_expr_refs(&attr.expr, path, from_addr, references, bound)
            }
            Structure::Block(block) if block.identifier.as_str() == "dynamic" => {
                collect_dynamic_refs(block, path, from_addr, references, bound);
            }
            Structure::Block(block) => {
                collect_body_refs(&block.body, path, from_addr, references, bound)
            }
        }
    }
}

/// Walk a `dynamic` block. The iterator is only in scope inside `content` (and
/// the `labels`/`iterator` attributes) — the `for_each` collection is evaluated
/// in the outer scope, so it keeps the outer `bound`.
fn collect_dynamic_refs(
    block: &Block,
    path: &Path,
    from_addr: &str,
    references: &mut Vec<TerraformRef>,
    bound: &[&str],
) {
    let mut inner = bound.to_vec();
    if let Some(iterator) = dynamic_iterator(block) {
        inner.push(iterator);
    }
    for structure in block.body.iter() {
        match structure {
            Structure::Attribute(attr) if attr.key.as_str() == "for_each" => {
                push_expr_refs(&attr.expr, path, from_addr, references, bound);
            }
            Structure::Attribute(attr) => {
                push_expr_refs(&attr.expr, path, from_addr, references, &inner);
            }
            Structure::Block(content) => {
                collect_body_refs(&content.body, path, from_addr, references, &inner);
            }
        }
    }
}

/// The iterator name a `dynamic` block binds in its `content`: the explicit
/// `iterator = name` if present, otherwise the block label.
fn dynamic_iterator(block: &Block) -> Option<&str> {
    for structure in block.body.iter() {
        if let Structure::Attribute(attr) = structure {
            if attr.key.as_str() == "iterator" {
                if let Expression::Variable(name) = &attr.expr {
                    return Some(name.as_str());
                }
            }
        }
    }
    block.labels.first().map(|label| label.as_str())
}

pub(super) fn push_expr_refs(
    expr: &Expression,
    path: &Path,
    from_addr: &str,
    references: &mut Vec<TerraformRef>,
    bound: &[&str],
) {
    let mut sink = Vec::new();
    walk_expr(expr, &mut sink, bound);
    for (to_addr, module_output) in sink {
        if to_addr == from_addr {
            continue;
        }
        references.push(TerraformRef {
            from_file: path.to_path_buf(),
            from_addr: from_addr.to_string(),
            to_addr,
            module_output,
        });
    }
}

/// Recursively collect referenced addresses from an expression. `bound` holds the
/// iterator variable names of any enclosing `for` expression/directive, which are
/// locals rather than resource references.
pub(super) fn walk_expr(
    expr: &Expression,
    sink: &mut Vec<(TfAddr, Option<String>)>,
    bound: &[&str],
) {
    match expr {
        Expression::Traversal(traversal) => {
            if !traversal_base_is_bound(traversal, bound) {
                if let Some(reference) = traversal_to_addr(traversal) {
                    sink.push(reference);
                }
            }
            // The base expression and any index expressions may hold more refs.
            walk_expr(&traversal.expr, sink, bound);
            for operator in &traversal.operators {
                if let TraversalOperator::Index(index) = operator {
                    walk_expr(index, sink, bound);
                }
            }
        }
        Expression::Array(items) => items.iter().for_each(|item| walk_expr(item, sink, bound)),
        Expression::Object(object) => walk_object(object, sink, bound),
        Expression::TemplateExpr(template_expr) => {
            if let Ok(template) = Template::from_expr(template_expr) {
                walk_template(&template, sink, bound);
            }
        }
        Expression::FuncCall(call) => call.args.iter().for_each(|arg| walk_expr(arg, sink, bound)),
        Expression::Parenthesis(inner) => walk_expr(inner, sink, bound),
        Expression::Conditional(conditional) => {
            walk_expr(&conditional.cond_expr, sink, bound);
            walk_expr(&conditional.true_expr, sink, bound);
            walk_expr(&conditional.false_expr, sink, bound);
        }
        Expression::Operation(operation) => match operation.as_ref() {
            Operation::Unary(unary) => walk_expr(&unary.expr, sink, bound),
            Operation::Binary(binary) => {
                walk_expr(&binary.lhs_expr, sink, bound);
                walk_expr(&binary.rhs_expr, sink, bound);
            }
        },
        Expression::ForExpr(for_expr) => {
            // The collection is evaluated in the outer scope; the key/value/cond
            // expressions see the iterator variables as locals.
            walk_expr(&for_expr.collection_expr, sink, bound);
            let mut inner = bound.to_vec();
            inner.push(for_expr.value_var.as_str());
            if let Some(key_var) = &for_expr.key_var {
                inner.push(key_var.as_str());
            }
            if let Some(key_expr) = &for_expr.key_expr {
                walk_expr(key_expr, sink, &inner);
            }
            walk_expr(&for_expr.value_expr, sink, &inner);
            if let Some(cond_expr) = &for_expr.cond_expr {
                walk_expr(cond_expr, sink, &inner);
            }
        }
        // Variable, Null, Bool, Number, String hold no traversal references.
        _ => {}
    }
}

fn traversal_base_is_bound(traversal: &Traversal, bound: &[&str]) -> bool {
    matches!(&traversal.expr, Expression::Variable(base) if bound.contains(&base.as_str()))
}

fn walk_object(
    object: &Object<ObjectKey, Expression>,
    sink: &mut Vec<(TfAddr, Option<String>)>,
    bound: &[&str],
) {
    for (key, value) in object.iter() {
        if let ObjectKey::Expression(expr) = key {
            walk_expr(expr, sink, bound);
        }
        walk_expr(value, sink, bound);
    }
}

fn walk_template(template: &Template, sink: &mut Vec<(TfAddr, Option<String>)>, bound: &[&str]) {
    for element in template.elements() {
        match element {
            Element::Interpolation(interpolation) => walk_expr(&interpolation.expr, sink, bound),
            Element::Directive(directive) => walk_directive(directive, sink, bound),
            Element::Literal(_) => {}
        }
    }
}

fn walk_directive(directive: &Directive, sink: &mut Vec<(TfAddr, Option<String>)>, bound: &[&str]) {
    match directive {
        Directive::If(if_directive) => {
            walk_expr(&if_directive.cond_expr, sink, bound);
            walk_template(&if_directive.true_template, sink, bound);
            if let Some(false_template) = &if_directive.false_template {
                walk_template(false_template, sink, bound);
            }
        }
        Directive::For(for_directive) => {
            walk_expr(&for_directive.collection_expr, sink, bound);
            let mut inner = bound.to_vec();
            inner.push(for_directive.value_var.as_str());
            if let Some(key_var) = &for_directive.key_var {
                inner.push(key_var.as_str());
            }
            walk_template(&for_directive.template, sink, &inner);
        }
    }
}
