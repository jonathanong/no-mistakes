use super::checkout::CheckoutState;
use super::diff::Form;
use super::revisions::Revision;
use super::verdicts::Verdict;

pub(super) const TWO_DOT_PR: &str = "diffs the pull request base against the head with a two-dot or \
    two-argument range, which also lists base-branch changes made after the branch point. \
    Use a three-dot diff (`git diff <base>...<head>`) after fetching enough history, or \
    `git diff HEAD^1 HEAD` on the merge commit";

pub(super) const MISMATCH_BASE_MERGE: &str = "pairs a pull request base revision with the merge commit \
    (`github.sha` or `HEAD`). Those revisions belong to different pairs and can include \
    unrelated base-branch commits even in a three-dot diff. Use `HEAD^1` with `HEAD` on the \
    merge commit, or pair `github.event.pull_request.base.sha` with `github.event.pull_request.head.sha`";

pub(super) const MISMATCH_BASEREF_MERGE_FORM: &str = "pairs `github.base_ref` with `github.sha` or `HEAD` \
    without a pull-request-only three-dot merge-base diff. Use `git diff HEAD^1 HEAD` on the \
    merge commit, or `origin/${{ github.base_ref }}...${{ github.event.pull_request.head.sha }}`";

pub(super) const MISMATCH_MERGE_PARENT_HEAD: &str = "pairs merge-commit parent `HEAD^1` with a non-merge \
    pull request head revision. Use `HEAD^1` with `HEAD` on a merge-commit checkout, or pair \
    `github.event.pull_request.base.sha` with `github.event.pull_request.head.sha`";

pub(super) const MISMATCH_CHECKOUT_MERGE_PARENT: &str = "diffs `HEAD^1` against `HEAD` after checking out \
    a non-merge revision (`actions/checkout` `with.ref`). On a PR head checkout, `HEAD^1` only \
    diffs the last commit instead of the whole pull request. Check out the merge commit or use a \
    three-dot diff between the PR base and head SHAs";

pub(super) const MISMATCH_CROSS_EVENT: &str = "mixes `pull_request` and `merge_group` revisions in one pair. \
    Pair `pull_request` base and head revisions together, `merge_group.base_sha` with \
    `merge_group.head_sha`, or matching `||` fallbacks on both ends";

pub(super) const MISMATCH_SAME_SIDE: &str = "pairs two revisions from the same side (base with base or head \
    with head) instead of a consistent base and head/merge pair";

pub(super) const UNKNOWN_CHECKOUT: &str = "uses `HEAD` or `HEAD^1` after an `actions/checkout` step whose \
    `with.ref` cannot be classified statically; use a literal merge-commit checkout or explicit \
    base/head SHAs, or suppress this line";

pub(super) const UNKNOWN_REVISION: &str = "uses a revision expression or variable that cannot be classified \
    statically, so base/merge pair consistency cannot be proven; pass explicit GitHub event SHAs \
    or `HEAD^1`/`HEAD`, or suppress this line";

pub(super) fn uses_unknown_checkout(a: Revision, b: Revision, checkout: CheckoutState) -> bool {
    let uses_head = matches!(a, Revision::CheckedOutUnknownHead | Revision::MergeCommitBase)
        || matches!(b, Revision::CheckedOutUnknownHead | Revision::MergeCommitBase);
    uses_head && checkout == CheckoutState::Unknown
}

pub(super) fn merge_parent_verdict(
    a: Revision,
    b: Revision,
    mixed: bool,
    checkout: CheckoutState,
) -> Option<Verdict> {
    let other = if matches!(a, Revision::MergeCommitBase) {
        b
    } else if matches!(b, Revision::MergeCommitBase) {
        a
    } else {
        return None;
    };
    if matches!(other, Revision::CheckedOutPrHead | Revision::CheckedOutPrBase)
        || matches!(
            checkout,
            CheckoutState::PullRequestHead | CheckoutState::PullRequestBase
        )
    {
        return Some(Verdict::Mismatch(MISMATCH_CHECKOUT_MERGE_PARENT));
    }
    let ok = checkout == CheckoutState::MergeCommit
        && (matches!(
            other,
            Revision::MergeCommitHead | Revision::MergeCommitSecondParent
        ) || (!mixed && matches!(other, Revision::GithubSha)));
    Some(if ok {
        Verdict::Consistent
    } else {
        Verdict::Mismatch(MISMATCH_MERGE_PARENT_HEAD)
    })
}

pub(super) fn merge_group_verdict(
    a: Revision,
    b: Revision,
    pull: bool,
    queue: bool,
) -> Option<Verdict> {
    let pair = [a, b];
    let has_mg_base = pair.contains(&Revision::MergeGroupBaseSha);
    let has_mg_head = pair.contains(&Revision::MergeGroupHeadSha);
    let has_pr_base = pair
        .iter()
        .any(|r| matches!(r, Revision::PullRequestBaseSha | Revision::PullRequestBaseRef));
    let has_pr_head = pair
        .iter()
        .any(|r| matches!(r, Revision::PullRequestHeadSha | Revision::PullRequestHeadRef));
    if (has_mg_base && has_pr_head)
        || (has_pr_base && has_mg_head)
        || (has_mg_base && pair.contains(&Revision::GithubSha) && pull && queue)
    {
        return Some(Verdict::Mismatch(MISMATCH_CROSS_EVENT));
    }
    let ok = (has_mg_base && has_mg_head)
        || (queue
            && !pull
            && has_mg_base
            && pair
                .iter()
                .any(|r| matches!(r, Revision::GithubSha | Revision::CheckedOutMergeGroupHead)));
    ok.then_some(Verdict::Consistent)
}

pub(super) fn has_coalesced_mismatch(a: Revision, b: Revision) -> bool {
    let a_coal = matches!(a, Revision::CoalescedBaseSha | Revision::CoalescedHeadSha);
    let b_coal = matches!(b, Revision::CoalescedBaseSha | Revision::CoalescedHeadSha);
    a_coal != b_coal
}

pub(super) fn base_with_merge_verdict(
    a: Revision,
    b: Revision,
    form: Option<Form>,
    mixed: bool,
    checkout: CheckoutState,
) -> Option<Verdict> {
    let pair = [a, b];
    let has_merge = pair
        .iter()
        .any(|r| matches!(r, Revision::GithubSha | Revision::MergeCommitHead));
    if !has_merge {
        return None;
    }
    if pair
        .iter()
        .any(|r| matches!(r, Revision::PullRequestBaseSha | Revision::CoalescedBaseSha))
    {
        return Some(Verdict::Mismatch(MISMATCH_BASE_MERGE));
    }
    if pair.contains(&Revision::PullRequestBaseRef) {
        let allowed = !mixed && form == Some(Form::ThreeDot) && checkout == CheckoutState::MergeCommit;
        return Some(if allowed {
            Verdict::Consistent
        } else if form.is_some() {
            Verdict::Mismatch(MISMATCH_BASEREF_MERGE_FORM)
        } else {
            Verdict::Mismatch(MISMATCH_BASE_MERGE)
        });
    }
    None
}

pub(super) fn is_same_side(a: Revision, b: Revision) -> bool {
    let is_base = |r: Revision| {
        r.is_pr_base() || matches!(r, Revision::MergeGroupBaseSha | Revision::CheckedOutPrBase)
    };
    let is_head = |r: Revision| {
        r.is_pr_head()
            || matches!(
                r,
                Revision::GithubSha
                    | Revision::MergeCommitHead
                    | Revision::MergeGroupHeadSha
                    | Revision::CheckedOutMergeGroupHead
            )
    };
    (is_base(a) && is_base(b)) || (is_head(a) && is_head(b))
}
