use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-pinned-hash";

#[test]
fn github_actions_pinned_hash_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_pinned_hash",
        &[
            (
                "pass",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "fail",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "local-action-pass",
                rules_path(RULE, "fixture", "local-action-pass"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "fail-excluded",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "exclude-ci.yml"),
            ),
            (
                "pinned-version-comment",
                rules_path(RULE, "fixture", "cases/pinned-version-comment"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "double-quoted-uses",
                rules_path(RULE, "fixture", "cases/double-quoted-uses"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "single-quoted-uses",
                rules_path(RULE, "fixture", "cases/single-quoted-uses"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "double-quoted-tag",
                rules_path(RULE, "fixture", "cases/double-quoted-tag"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "build-metadata-comment",
                rules_path(RULE, "fixture", "cases/build-metadata-comment"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "uppercase-sha",
                rules_path(RULE, "fixture", "cases/uppercase-sha"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "tag-ref",
                rules_path(RULE, "fixture", "cases/tag-ref"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "branch-ref",
                rules_path(RULE, "fixture", "cases/branch-ref"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "sha-without-comment",
                rules_path(RULE, "fixture", "cases/sha-without-comment"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "sha-bad-comment",
                rules_path(RULE, "fixture", "cases/sha-bad-comment"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "major-only-comment",
                rules_path(RULE, "fixture", "cases/major-only-comment"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "numeric-version-comment",
                rules_path(RULE, "fixture", "cases/numeric-version-comment"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "local-action",
                rules_path(RULE, "fixture", "cases/local-action"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "docker-ref",
                rules_path(RULE, "fixture", "cases/docker-ref"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "comment-uses",
                rules_path(RULE, "fixture", "cases/comment-uses"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "short-sha",
                rules_path(RULE, "fixture", "cases/short-sha"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "reusable-workflow",
                rules_path(RULE, "fixture", "cases/reusable-workflow"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "shapes-anchor-flow-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/anchor-flow-fail.yml"),
            ),
            (
                "shapes-job-named-env-workflow-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/job-named-env-workflow-fail.yml"),
            ),
            (
                "shapes-anchored-env-uses-pass",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/anchored-env-uses-pass.yml"),
            ),
            (
                "shapes-mapping-alias-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/mapping-alias-fail.yml"),
            ),
            (
                "shapes-aliased-steps-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/aliased-steps-fail.yml"),
            ),
            (
                "shapes-dedup-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/dedup-fail.yml"),
            ),
            (
                "shapes-anchored-flow-steps-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/anchored-flow-steps-fail.yml"),
            ),
            (
                "shapes-escaped-comment-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/escaped-comment-fail.yml"),
            ),
            (
                "shapes-flow-break-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/flow-break-fail.yml"),
            ),
            (
                "shapes-folded-flow-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/folded-flow-fail.yml"),
            ),
            (
                "shapes-next-line-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/next-line-fail.yml"),
            ),
            (
                "shapes-numeric-job-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/numeric-job-fail.yml"),
            ),
            (
                "shapes-with-comment-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/with-comment-fail.yml"),
            ),
            (
                "shapes-next-line-pass",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/next-line-pass.yml"),
            ),
            (
                "shapes-job-named-env-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/job-named-env-fail.yml"),
            ),
            (
                "shapes-flow-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/flow-fail.yml"),
            ),
            (
                "shapes-job-named-steps-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/job-named-steps-fail.yml"),
            ),
            (
                "shapes-echo-ref-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/echo-ref-fail.yml"),
            ),
            (
                "shapes-nested-uses-pass",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/nested-uses-pass.yml"),
            ),
            (
                "shapes-quote-escape-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/quote-escape-fail.yml"),
            ),
            (
                "shapes-anchor-key-same-line-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/anchor-key-same-line-fail.yml"),
            ),
            (
                "shapes-alias-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/alias-fail.yml"),
            ),
            (
                "shapes-block-script-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/block-script-fail.yml"),
            ),
            (
                "shapes-anchor-step-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/anchor-step-fail.yml"),
            ),
            (
                "shapes-flow-with-break-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/flow-with-break-fail.yml"),
            ),
            (
                "shapes-plain-hash-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/plain-hash-fail.yml"),
            ),
            (
                "shapes-folded-quote-comment-pass",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/folded-quote-comment-pass.yml"),
            ),
            (
                "shapes-quoted-key-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/quoted-key-fail.yml"),
            ),
            (
                "shapes-block-chomp-comment-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/block-chomp-comment-fail.yml"),
            ),
            (
                "shapes-comment-before-value-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/comment-before-value-fail.yml"),
            ),
            (
                "shapes-space-before-colon-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/space-before-colon-fail.yml"),
            ),
            (
                "shapes-same-line-uses-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/same-line-uses-fail.yml"),
            ),
            (
                "shapes-matrix-next-line-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/matrix-next-line-fail.yml"),
            ),
            (
                "shapes-multi-doc-flow-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/multi-doc-flow-fail.yml"),
            ),
            (
                "shapes-plain-apostrophe-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/plain-apostrophe-fail.yml"),
            ),
            (
                "shapes-plain-apostrophe-comment-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/plain-apostrophe-comment-fail.yml"),
            ),
            (
                "shapes-quoted-escape-key-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/quoted-escape-key-fail.yml"),
            ),
            (
                "shapes-bare-dash-step-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/bare-dash-step-fail.yml"),
            ),
            (
                "shapes-block-header-comment-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/block-header-comment-fail.yml"),
            ),
            (
                "shapes-same-indent-steps-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/same-indent-steps-fail.yml"),
            ),
            (
                "shapes-multi-doc-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/multi-doc-fail.yml"),
            ),
            (
                "shapes-matrix-uses-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/matrix-uses-fail.yml"),
            ),
            (
                "shapes-flow-pass",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/flow-pass.yml"),
            ),
            (
                "shapes-with-uses-pass",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/with-uses-pass.yml"),
            ),
            (
                "shapes-flow-hash-comment-fail",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/flow-hash-comment-fail.yml"),
            ),
            (
                "shapes-with-block-uses-pass",
                rules_path(RULE, "fixture", "yaml-shapes"),
                rules_path(RULE, "configs", "shapes/with-block-uses-pass.yml"),
            ),
        ],
    );
}
