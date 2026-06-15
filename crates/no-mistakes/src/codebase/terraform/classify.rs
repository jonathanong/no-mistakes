//! Classification of HCL traversals into declarable Terraform addresses. Split
//! from `references.rs` to keep each source file within the per-file line budget.

use hcl::expr::{Expression, Traversal, TraversalOperator};

use super::TfAddr;

/// Base traversal identifiers that are Terraform meta-values, not references.
const META_BASES: &[&str] = &["count", "each", "self", "path", "terraform"];

/// Classify a traversal into a declarable address, plus an optional module output.
pub(super) fn traversal_to_addr(traversal: &Traversal) -> Option<(TfAddr, Option<String>)> {
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

/// Collect the leading `GetAttr` identifiers of a traversal. Index operators are
/// skipped so indexed resources/modules (`module.net[0].out`,
/// `aws_x.y[count.index].id`) still resolve their name and output; splat operators
/// terminate the chain.
fn leading_attrs(operators: &[TraversalOperator]) -> Vec<&str> {
    let mut attrs = Vec::new();
    for operator in operators {
        match operator {
            TraversalOperator::GetAttr(ident) => attrs.push(ident.as_str()),
            TraversalOperator::Index(_) | TraversalOperator::LegacyIndex(_) => continue,
            _ => break,
        }
    }
    attrs
}
