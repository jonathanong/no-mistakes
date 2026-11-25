use super::checkout::CheckoutState;
use super::diff::Form;
use super::revisions::Revision;
use super::verdicts_helpers::{
    check_base_with_merge, check_cross_event, check_merge_parent, has_coalesced_mismatch,
    is_baseref_merge_allowed, is_merge_group_consistent, is_merge_parent_consistent, is_same_side,
    uses_unknown_checkout, MISMATCH_BASE_MERGE, MISMATCH_CROSS_EVENT, MISMATCH_SAME_SIDE,
    TWO_DOT_PR, UNKNOWN_CHECKOUT, UNKNOWN_REVISION,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    Consistent,
    Mismatch(&'static str),
    Unanalyzable(&'static str),
}

pub(super) fn judge_diff_pair(
    left: Revision,
    right: Revision,
    form: Form,
    mixed: bool,
    pull: bool,
    queue: bool,
    checkout: CheckoutState,
) -> Verdict {
    if !left.is_known() && !right.is_known() {
        return Verdict::Consistent;
    }
    if !left.is_known() || !right.is_known() {
        return Verdict::Unanalyzable(UNKNOWN_REVISION);
    }
    if uses_unknown_checkout(left, right, checkout) {
        return Verdict::Unanalyzable(UNKNOWN_CHECKOUT);
    }
    if let Some(msg) = check_merge_parent(left, right, mixed, checkout) {
        return Verdict::Mismatch(msg);
    }
    if is_merge_parent_consistent(left, right, mixed, checkout) {
        return Verdict::Consistent;
    }
    if let Some(msg) = check_cross_event(left, right, pull, queue) {
        return Verdict::Mismatch(msg);
    }
    if is_merge_group_consistent(left, right, pull, queue) {
        return Verdict::Consistent;
    }
    if (left.is_pr_base() && right.is_pr_head()) || (right.is_pr_base() && left.is_pr_head()) {
        if has_coalesced_mismatch(left, right) {
            return Verdict::Mismatch(MISMATCH_CROSS_EVENT);
        }
        return if form == Form::ThreeDot {
            Verdict::Consistent
        } else {
            Verdict::Mismatch(TWO_DOT_PR)
        };
    }
    if let Some(msg) = check_base_with_merge(left, right, Some(form), mixed) {
        return Verdict::Mismatch(msg);
    }
    if is_baseref_merge_allowed(left, right, form, mixed, checkout) {
        return Verdict::Consistent;
    }
    if is_same_side(left, right) {
        return Verdict::Mismatch(MISMATCH_SAME_SIDE);
    }
    Verdict::Consistent
}

pub(super) fn judge_explicit_pair(
    base: Revision,
    head: Revision,
    mixed: bool,
    pull: bool,
    queue: bool,
    checkout: CheckoutState,
) -> Verdict {
    if !base.is_known() || !head.is_known() {
        return Verdict::Unanalyzable(UNKNOWN_REVISION);
    }
    if uses_unknown_checkout(base, head, checkout) {
        return Verdict::Unanalyzable(UNKNOWN_CHECKOUT);
    }
    if let Some(msg) = check_merge_parent(base, head, mixed, checkout) {
        return Verdict::Mismatch(msg);
    }
    if is_merge_parent_consistent(base, head, mixed, checkout) {
        return Verdict::Consistent;
    }
    if let Some(msg) = check_cross_event(base, head, pull, queue) {
        return Verdict::Mismatch(msg);
    }
    if is_merge_group_consistent(base, head, pull, queue) {
        return Verdict::Consistent;
    }
    if base.is_pr_base() && head.is_pr_head() {
        return if has_coalesced_mismatch(base, head) {
            Verdict::Mismatch(MISMATCH_CROSS_EVENT)
        } else {
            Verdict::Consistent
        };
    }
    if let Some(msg) = check_base_with_merge(base, head, None, mixed) {
        return Verdict::Mismatch(msg);
    }
    if is_same_side(base, head) {
        return Verdict::Mismatch(MISMATCH_SAME_SIDE);
    }
    Verdict::Mismatch(MISMATCH_BASE_MERGE)
}
