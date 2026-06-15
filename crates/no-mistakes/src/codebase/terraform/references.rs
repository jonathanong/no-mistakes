//! Reference extraction from HCL expressions: walks expression trees and
//! classifies traversals into declarable Terraform addresses. Split from
//! `parse.rs` to keep each source file within the per-file line budget.

use hcl::expr::{Expression, Object, ObjectKey, Operation, Traversal, TraversalOperator};
use hcl::structure::{Body, Structure};
use hcl::template::{Directive, Element, Template};
use std::path::Path;

use super::{TerraformRef, TfAddr};

/// Base traversal identifiers that are Terraform meta-values, not references.
const META_BASES: &[&str] = &["count", "each", "self", "path", "terraform"];

/// Walk every attribute in a block body (recursing into nested blocks) and record
/// references attributed to the enclosing block address.
pub(super) fn collect_body_refs(
    body: &Body,
    path: &Path,
    from_addr: &str,
    references: &mut Vec<TerraformRef>,
) {
    for structure in body.iter() {
        match structure {
            Structure::Attribute(attr) => push_expr_refs(&attr.expr, path, from_addr, references),
            Structure::Block(block) => collect_body_refs(&block.body, path, from_addr, references),
        }
    }
}

pub(super) fn push_expr_refs(
    expr: &Expression,
    path: &Path,
    from_addr: &str,
    references: &mut Vec<TerraformRef>,
) {
    let mut sink = Vec::new();
    walk_expr(expr, &mut sink);
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

/// Recursively collect referenced addresses from an expression.
pub(super) fn walk_expr(expr: &Expression, sink: &mut Vec<(TfAddr, Option<String>)>) {
    match expr {
        Expression::Traversal(traversal) => {
            if let Some(reference) = traversal_to_addr(traversal) {
                sink.push(reference);
            }
            // The base expression and any index expressions may hold more refs.
            walk_expr(&traversal.expr, sink);
            for operator in &traversal.operators {
                if let TraversalOperator::Index(index) = operator {
                    walk_expr(index, sink);
                }
            }
        }
        Expression::Array(items) => items.iter().for_each(|item| walk_expr(item, sink)),
        Expression::Object(object) => walk_object(object, sink),
        Expression::TemplateExpr(template_expr) => {
            if let Ok(template) = Template::from_expr(template_expr) {
                walk_template(&template, sink);
            }
        }
        Expression::FuncCall(call) => call.args.iter().for_each(|arg| walk_expr(arg, sink)),
        Expression::Parenthesis(inner) => walk_expr(inner, sink),
        Expression::Conditional(conditional) => {
            walk_expr(&conditional.cond_expr, sink);
            walk_expr(&conditional.true_expr, sink);
            walk_expr(&conditional.false_expr, sink);
        }
        Expression::Operation(operation) => match operation.as_ref() {
            Operation::Unary(unary) => walk_expr(&unary.expr, sink),
            Operation::Binary(binary) => {
                walk_expr(&binary.lhs_expr, sink);
                walk_expr(&binary.rhs_expr, sink);
            }
        },
        Expression::ForExpr(for_expr) => {
            walk_expr(&for_expr.collection_expr, sink);
            if let Some(key_expr) = &for_expr.key_expr {
                walk_expr(key_expr, sink);
            }
            walk_expr(&for_expr.value_expr, sink);
            if let Some(cond_expr) = &for_expr.cond_expr {
                walk_expr(cond_expr, sink);
            }
        }
        // Variable, Null, Bool, Number, String hold no traversal references.
        _ => {}
    }
}

fn walk_object(object: &Object<ObjectKey, Expression>, sink: &mut Vec<(TfAddr, Option<String>)>) {
    for (key, value) in object.iter() {
        if let ObjectKey::Expression(expr) = key {
            walk_expr(expr, sink);
        }
        walk_expr(value, sink);
    }
}

fn walk_template(template: &Template, sink: &mut Vec<(TfAddr, Option<String>)>) {
    for element in template.elements() {
        match element {
            Element::Interpolation(interpolation) => walk_expr(&interpolation.expr, sink),
            Element::Directive(directive) => walk_directive(directive, sink),
            Element::Literal(_) => {}
        }
    }
}

fn walk_directive(directive: &Directive, sink: &mut Vec<(TfAddr, Option<String>)>) {
    match directive {
        Directive::If(if_directive) => {
            walk_expr(&if_directive.cond_expr, sink);
            walk_template(&if_directive.true_template, sink);
            if let Some(false_template) = &if_directive.false_template {
                walk_template(false_template, sink);
            }
        }
        Directive::For(for_directive) => {
            walk_expr(&for_directive.collection_expr, sink);
            walk_template(&for_directive.template, sink);
        }
    }
}

/// Classify a traversal into a declarable address, plus an optional module output.
fn traversal_to_addr(traversal: &Traversal) -> Option<(TfAddr, Option<String>)> {
    let Expression::Variable(base) = &traversal.expr else {
        return None;
    };
    let base = base.as_str();
    if META_BASES.contains(&base) {
        return None;
    }
    let attrs = leading_attrs(&traversal.operators);
    match base {
        "var" => attrs.first().map(|name| (format!("var.{name}"), None)),
        "local" => attrs.first().map(|name| (format!("local.{name}"), None)),
        "data" => match (attrs.first(), attrs.get(1)) {
            (Some(type_label), Some(name)) => Some((format!("data.{type_label}.{name}"), None)),
            _ => None,
        },
        "module" => attrs.first().map(|name| {
            (
                format!("module.{name}"),
                attrs.get(1).map(|output| (*output).to_string()),
            )
        }),
        // Anything else with at least one attribute is a resource reference.
        _ => attrs.first().map(|name| (format!("{base}.{name}"), None)),
    }
}

/// Collect the leading `GetAttr` identifiers of a traversal, stopping at the first
/// non-attribute operator (index, splat).
fn leading_attrs(operators: &[TraversalOperator]) -> Vec<&str> {
    let mut attrs = Vec::new();
    for operator in operators {
        match operator {
            TraversalOperator::GetAttr(ident) => attrs.push(ident.as_str()),
            _ => break,
        }
    }
    attrs
}
