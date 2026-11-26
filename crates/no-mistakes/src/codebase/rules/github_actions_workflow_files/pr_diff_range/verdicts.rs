use super::checkout::CheckoutState;
use super::diff::Form;
use super::revisions::Revision;
use super::verdicts_helpers::{
    base_with_merge_verdict, has_coalesced_mismatch, is_same_side, merge_group_verdict,
    merge_parent_verdict, uses_unknown_checkout, MISMATCH_BASE_MERGE, MISMATCH_CROSS_EVENT,
    MISMATCH_SAME_SIDE, TWO_DOT_PR, UNKNOWN_CHECKOUT, UNKNOWN_REVISION,
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
    judge_pair(left, right, Some(form), mixed, pull, queue, checkout)
}

pub(super) fn judge_explicit_pair(
    base: Revision,
    head: Revision,
    mixed: bool,
    pull: bool,
    queue: bool,
    checkout: CheckoutState,
) -> Verdict {
    judge_pair(base, head, None, mixed, pull, queue, checkout)
}

fn judge_pair(
    a: Revision,
    b: Revision,
    form: Option<Form>,
    mixed: bool,
    pull: bool,
    queue: bool,
    checkout: CheckoutState,
) -> Verdict {
    if !a.is_known() || !b.is_known() {
        return Verdict::Unanalyzable(UNKNOWN_REVISION);
    }
    if uses_unknown_checkout(a, b, checkout) {
        return Verdict::Unanalyzable(UNKNOWN_CHECKOUT);
    }
    if let Some(verdict) = merge_parent_verdict(a, b, mixed, checkout) {
        return verdict;
    }
    if let Some(verdict) = merge_group_verdict(a, b, pull, queue) {
        return verdict;
    }
    let pr_pair = if form.is_some() {
        (a.is_pr_base() && b.is_pr_head()) || (b.is_pr_base() && a.is_pr_head())
    } else {
        a.is_pr_base() && b.is_pr_head()
    };
    if pr_pair {
        if has_coalesced_mismatch(a, b) {
            return Verdict::Mismatch(MISMATCH_CROSS_EVENT);
        }
        return match form {
            None | Some(Form::ThreeDot) => Verdict::Consistent,
            Some(Form::TwoDot | Form::Args) => Verdict::Mismatch(TWO_DOT_PR),
        };
    }
    if let Some(verdict) = base_with_merge_verdict(a, b, form, mixed, checkout) {
        return verdict;
    }
    if is_same_side(a, b) {
        return Verdict::Mismatch(MISMATCH_SAME_SIDE);
    }
    Verdict::Mismatch(MISMATCH_BASE_MERGE)
}
