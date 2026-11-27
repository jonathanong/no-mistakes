use super::checkout::CheckoutState;
use super::exprs::{extract, variable};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Revision {
    PullRequestBaseSha,
    PullRequestBaseRef,
    PullRequestHeadSha,
    PullRequestHeadRef,
    GithubSha,
    MergeCommitBase,
    MergeCommitSecondParent,
    MergeCommitHead,
    CheckedOutPrHead,
    CheckedOutPrBase,
    CheckedOutMergeGroupHead,
    CheckedOutUnknownHead,
    MergeGroupBaseSha,
    MergeGroupHeadSha,
    CoalescedBaseSha,
    CoalescedHeadSha,
    Unknown,
}

impl Revision {
    pub(super) fn is_known(self) -> bool {
        !matches!(self, Self::Unknown)
    }

    pub(super) fn is_pr_base(self) -> bool {
        matches!(
            self,
            Self::PullRequestBaseSha | Self::PullRequestBaseRef | Self::CoalescedBaseSha
        )
    }

    pub(super) fn is_pr_head(self) -> bool {
        matches!(
            self,
            Self::PullRequestHeadSha
                | Self::PullRequestHeadRef
                | Self::CoalescedHeadSha
                | Self::CheckedOutPrHead
        )
    }
}

pub(super) fn classify_value(raw: &str, checkout: CheckoutState) -> Revision {
    let trimmed = raw.trim();
    let (text, exprs) = extract(trimmed);
    classify_literal(&text, &exprs, checkout)
}

pub(super) fn classify_token(
    piece: &str,
    exprs: &[String],
    env: &BTreeMap<String, String>,
    checkout: CheckoutState,
) -> Revision {
    let piece = piece.trim_matches(|ch| ch == '"' || ch == '\'');
    if let Some(name) = variable(piece) {
        let Some(value) = env.get(name) else {
            return Revision::Unknown;
        };
        return classify_value(value, checkout);
    }
    classify_literal(piece, exprs, checkout)
}

fn classify_literal(piece: &str, exprs: &[String], checkout: CheckoutState) -> Revision {
    match piece {
        "HEAD^1" | "HEAD^" | "HEAD~1" | "HEAD~" => return Revision::MergeCommitBase,
        "HEAD^2" => return Revision::MergeCommitSecondParent,
        "HEAD" => {
            return match checkout {
                CheckoutState::MergeCommit => Revision::MergeCommitHead,
                CheckoutState::PullRequestHead => Revision::CheckedOutPrHead,
                CheckoutState::PullRequestBase => Revision::CheckedOutPrBase,
                CheckoutState::MergeGroup => Revision::CheckedOutMergeGroupHead,
                CheckoutState::Unknown => Revision::CheckedOutUnknownHead,
            };
        }
        _ => {}
    }
    let stripped = piece
        .strip_prefix("refs/remotes/origin/")
        .or_else(|| piece.strip_prefix("origin/"))
        .unwrap_or(piece);
    let Some(expression) = stripped
        .strip_prefix("__ghx")
        .and_then(|rest| rest.strip_suffix("__"))
        .and_then(|digits| digits.parse::<usize>().ok())
        .and_then(|index| exprs.get(index))
    else {
        return Revision::Unknown;
    };
    match expression.as_str() {
        "github.event.pull_request.base.sha" => Revision::PullRequestBaseSha,
        "github.base_ref" | "github.event.pull_request.base.ref" => Revision::PullRequestBaseRef,
        "github.event.pull_request.head.sha" => Revision::PullRequestHeadSha,
        "github.head_ref" | "github.event.pull_request.head.ref" => Revision::PullRequestHeadRef,
        "github.sha" => Revision::GithubSha,
        "github.event.merge_group.base_sha" => Revision::MergeGroupBaseSha,
        "github.event.merge_group.head_sha" => Revision::MergeGroupHeadSha,
        other => classify_fallback(other).unwrap_or(Revision::Unknown),
    }
}

pub(super) fn classify_fallback(expression: &str) -> Option<Revision> {
    let (left, right) = expression.split_once("||")?;
    match (left.trim(), right.trim()) {
        ("github.event.pull_request.base.sha", "github.event.merge_group.base_sha")
        | ("github.event.merge_group.base_sha", "github.event.pull_request.base.sha") => {
            Some(Revision::CoalescedBaseSha)
        }
        ("github.event.pull_request.head.sha", "github.event.merge_group.head_sha")
        | ("github.event.merge_group.head_sha", "github.event.pull_request.head.sha") => {
            Some(Revision::CoalescedHeadSha)
        }
        _ => None,
    }
}
