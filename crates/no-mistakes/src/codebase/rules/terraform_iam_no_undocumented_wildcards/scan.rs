use super::expr::{
    classify_resource_item, is_local_policy_doc_ref, unparenthesize, ItemStatus,
    UnanalyzableReason,
};
use crate::fx::FxHashSet;
use hcl::edit::parser::{self, Error};
use hcl::edit::structure::{Attribute, Block};
use hcl::edit::Span;

const MERGED_POLICY_ATTRS: &[&str] = &[
    "source_policy_documents",
    "override_policy_documents",
    "source_json",
    "override_json",
];

pub(super) enum ScanHitKind {
    Wildcard,
    Unanalyzable(UnanalyzableReason),
}

pub(super) struct ScanHit {
    pub(super) offset: usize,
    pub(super) kind: ScanHitKind,
}

pub(super) fn scan_hcl(source: &str) -> Result<Vec<ScanHit>, Error> {
    let body = parser::parse_body(source)?;
    let local_docs: FxHashSet<&str> = body
        .get_blocks("data")
        .filter_map(|block| match block.labels.as_slice() {
            [data_type, name] if data_type.as_str() == "aws_iam_policy_document" => {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect();

    let mut hits = Vec::new();
    for block in body.get_blocks("data") {
        if matches!(
            block.labels.as_slice(),
            [data_type, _] if data_type.as_str() == "aws_iam_policy_document"
        ) {
            scan_policy_document(block, &mut hits);
        }
    }
    for block in body.get_blocks("resource") {
        scan_policy_resource(block, &local_docs, &mut hits);
    }
    hits.sort_by_key(|hit| hit.offset);
    Ok(hits)
}

fn scan_policy_document(block: &Block, hits: &mut Vec<ScanHit>) {
    for attr_name in MERGED_POLICY_ATTRS {
        if let Some(attr) = block.body.get_attribute(attr_name) {
            let value = unparenthesize(&attr.value);
            if !value.is_null() && !value.as_array().is_some_and(|arr| arr.is_empty()) {
                push_attr(hits, attr, ScanHitKind::Unanalyzable(UnanalyzableReason::MergedPolicyDocument));
            }
        }
    }
    for dynamic in block.body.get_blocks("dynamic") {
        if matches!(dynamic.labels.as_slice(), [label] if label.as_str() == "statement") {
            push_block(hits, dynamic, ScanHitKind::Unanalyzable(UnanalyzableReason::DynamicStatement));
        }
    }
    for statement in block.body.get_blocks("statement") {
        scan_statement(statement, hits);
    }
}

const IAM_POLICY_RESOURCES: &[&str] = &[
    "aws_iam_policy",
    "aws_iam_role_policy",
    "aws_iam_user_policy",
    "aws_iam_group_policy",
];

fn scan_policy_resource(block: &Block, local_docs: &FxHashSet<&str>, hits: &mut Vec<ScanHit>) {
    if let [res_type, _] = block.labels.as_slice() {
        let kind = res_type.as_str();
        if IAM_POLICY_RESOURCES.contains(&kind) {
            check_policy_attr(&block.body, local_docs, hits);
        } else if kind == "aws_iam_role" {
            scan_role_inline_policies(&block.body, local_docs, hits);
        }
    }
}

fn scan_role_inline_policies(
    body: &hcl::edit::structure::Body,
    local_docs: &FxHashSet<&str>,
    hits: &mut Vec<ScanHit>,
) {
    for inline in body.get_blocks("inline_policy") {
        check_policy_attr(&inline.body, local_docs, hits);
    }
    for dynamic in body.get_blocks("dynamic") {
        if matches!(dynamic.labels.as_slice(), [label] if label.as_str() == "inline_policy") {
            push_block(
                hits,
                dynamic,
                ScanHitKind::Unanalyzable(UnanalyzableReason::UnsupportedPolicyShape),
            );
        }
    }
}

fn check_policy_attr(
    body: &hcl::edit::structure::Body,
    local_docs: &FxHashSet<&str>,
    hits: &mut Vec<ScanHit>,
) {
    if let Some(policy) = body.get_attribute("policy") {
        if !is_local_policy_doc_ref(&policy.value, local_docs) {
            push_attr(
                hits,
                policy,
                ScanHitKind::Unanalyzable(UnanalyzableReason::UnsupportedPolicyShape),
            );
        }
    }
}

fn scan_statement(statement: &Block, hits: &mut Vec<ScanHit>) {
    if statement.is_labeled() {
        push_block(
            hits,
            statement,
            ScanHitKind::Unanalyzable(UnanalyzableReason::UnexpectedStatementLabel),
        );
        return;
    }
    if let Some(effect) = statement.body.get_attribute("effect") {
        match unparenthesize(&effect.value).as_str() {
            Some("Allow") => {}
            Some("Deny") => return,
            _ => {
                push_attr(
                    hits,
                    effect,
                    ScanHitKind::Unanalyzable(UnanalyzableReason::ComputedEffect),
                );
                return;
            }
        }
    }
    let Some(resources) = statement.body.get_attribute("resources") else {
        return;
    };
    let Some(items) = unparenthesize(&resources.value).as_array() else {
        push_attr(
            hits,
            resources,
            ScanHitKind::Unanalyzable(UnanalyzableReason::ComputedResources),
        );
        return;
    };
    let mut has_unanalyzable = false;
    for item in items.iter() {
        match classify_resource_item(unparenthesize(item)) {
            ItemStatus::Wildcard => {
                push_attr(hits, resources, ScanHitKind::Wildcard);
                return;
            }
            ItemStatus::Unanalyzable => has_unanalyzable = true,
            ItemStatus::Clean => {}
        }
    }
    if has_unanalyzable {
        push_attr(
            hits,
            resources,
            ScanHitKind::Unanalyzable(UnanalyzableReason::NonLiteralResource),
        );
    }
}

fn push_attr(hits: &mut Vec<ScanHit>, attr: &Attribute, kind: ScanHitKind) {
    if let Some(span) = attr.key.span() {
        hits.push(ScanHit {
            offset: span.start,
            kind,
        });
    }
}

fn push_block(hits: &mut Vec<ScanHit>, block: &Block, kind: ScanHitKind) {
    if let Some(span) = block.ident.span() {
        hits.push(ScanHit {
            offset: span.start,
            kind,
        });
    }
}
