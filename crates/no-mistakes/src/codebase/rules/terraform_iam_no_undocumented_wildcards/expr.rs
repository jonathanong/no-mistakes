use crate::fx::FxHashSet;
use hcl::edit::expr::{Expression, TraversalOperator};
use hcl::edit::template::{Element, Template};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UnanalyzableReason {
    ComputedEffect,
    ComputedResources,
    DynamicStatement,
    MergedPolicyDocument,
    NonLiteralResource,
    UnsupportedPolicyShape,
    UnexpectedStatementLabel,
}

impl UnanalyzableReason {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::ComputedEffect => {
                "Cannot statically verify IAM statement effect because it is not a string literal. Use literal \"Allow\" or \"Deny\", or add a suppression with the reason."
            }
            Self::ComputedResources => {
                "Cannot statically verify IAM resources because the attribute is not a literal list. Use a list of explicit resource ARNs, or add a suppression with the reason."
            }
            Self::DynamicStatement => {
                "Cannot statically verify dynamic IAM statement block for resource wildcards. Replace with explicit statement blocks, or add a suppression with the reason."
            }
            Self::MergedPolicyDocument => {
                "Cannot statically verify merged IAM policy documents for resource wildcards. Inline explicit statement blocks, or add a suppression with the reason."
            }
            Self::NonLiteralResource => {
                "Cannot statically verify non-literal IAM resource entry for wildcards. Use explicit resource ARNs, or add a suppression with the reason."
            }
            Self::UnsupportedPolicyShape => {
                "Cannot statically verify IAM policy attribute because it is not a local aws_iam_policy_document reference. Use a local aws_iam_policy_document data source, or add a suppression with the reason."
            }
            Self::UnexpectedStatementLabel => {
                "Cannot statically verify labeled IAM statement block. Use an unlabeled statement block, or add a suppression with the reason."
            }
        }
    }
}

pub(super) enum ItemStatus {
    Clean,
    Wildcard,
    Unanalyzable,
}

pub(super) fn classify_resource_item(expr: &Expression) -> ItemStatus {
    if let Some(value) = expr.as_str() {
        return if has_wildcard(value) {
            ItemStatus::Wildcard
        } else {
            ItemStatus::Clean
        };
    }
    if let Some(template) = expr.as_template() {
        return classify_template(template);
    }
    ItemStatus::Unanalyzable
}

fn classify_template(template: &Template) -> ItemStatus {
    let mut has_non_literal = false;
    for element in template.iter() {
        match element {
            Element::Literal(literal) => {
                if has_wildcard(literal.as_str()) {
                    return ItemStatus::Wildcard;
                }
            }
            Element::Interpolation(_) | Element::Directive(_) => has_non_literal = true,
        }
    }
    if has_non_literal {
        ItemStatus::Unanalyzable
    } else {
        ItemStatus::Clean
    }
}

pub(super) fn is_local_policy_doc_ref(expr: &Expression, local_docs: &FxHashSet<&str>) -> bool {
    let Expression::Traversal(traversal) = unparenthesize(expr) else {
        return false;
    };
    let Expression::Variable(root) = &traversal.expr else {
        return false;
    };
    if root.as_str() != "data" {
        return false;
    }
    let attrs: Vec<&str> = traversal
        .operators
        .iter()
        .filter_map(|op| match op.value() {
            TraversalOperator::GetAttr(ident) => Some(ident.as_str()),
            _ => None,
        })
        .collect();
    attrs.len() == traversal.operators.len()
        && matches!(
            attrs.as_slice(),
            ["aws_iam_policy_document", name, "json" | "minified_json"] if local_docs.contains(name)
        )
}

pub(super) fn has_wildcard(mut value: &str) -> bool {
    while let Some(index) = value.find(['*', '?', '$', '&']) {
        let tail = &value[index..];
        if tail.starts_with(['*', '?']) {
            return true;
        }
        if let Some(variable) = tail.strip_prefix("${").or_else(|| tail.strip_prefix("&{")) {
            let Some((_, rest)) = variable.split_once('}') else {
                return false;
            };
            value = rest;
        } else {
            value = &tail[1..];
        }
    }
    false
}

pub(super) fn unparenthesize(mut expression: &Expression) -> &Expression {
    while let Expression::Parenthesis(parenthesis) = expression {
        expression = parenthesis.inner();
    }
    expression
}
