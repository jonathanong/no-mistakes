use std::path::{Path, PathBuf};

#[test]
fn check_json_snapshot_rule_recall_audit() {
    let base =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rule-recall-audit/fixture");
    let cases = [
        ("workflow/file-suppression", "config.json"),
        ("workflow/line-suppression", "config.json"),
        ("workflow/missing-edge", "config.json"),
        ("workflow/edge-control", "config.json"),
        ("live-scan/missing-relative", ".no-mistakes.yml"),
        ("live-scan/missing-alias", ".no-mistakes.yml"),
        ("live-scan/owned", ".no-mistakes.yml"),
        ("live-scan/absolute-control", ".no-mistakes.yml"),
        ("storybook/empty-stories", ".no-mistakes.yml"),
        ("storybook/nonmatching-stories", ".no-mistakes.yml"),
        ("storybook/required-props-explicit", ".no-mistakes.yml"),
        ("storybook/required-props-kind", ".no-mistakes.yml"),
        ("download-integrity/substitution-check", ".no-mistakes.yml"),
        ("download-integrity/quoted-backtick", ".no-mistakes.yml"),
        (
            "download-integrity/unquoted-substitution",
            ".no-mistakes.yml",
        ),
        ("download-integrity/unsafe", ".no-mistakes.yml"),
        ("timeout-literals/multiline", ".no-mistakes.yml"),
        ("timeout-literals/single-line", ".no-mistakes.yml"),
        ("child-links/html-off", ".no-mistakes.yml"),
        ("child-links/missing", ".no-mistakes.yml"),
        ("child-links/html-on", ".no-mistakes.yml"),
        ("child-links/next-line", ".no-mistakes.yml"),
        ("child-links/file-disable", ".no-mistakes.yml"),
        ("storybook/omitted-stories", ".no-mistakes.yml"),
        ("storybook/required-props-matching", ".no-mistakes.yml"),
        ("workflow/inline-suppression", "config.json"),
        (
            "download-integrity/escaped-substitution",
            ".no-mistakes.yml",
        ),
        ("download-integrity/literal-check", ".no-mistakes.yml"),
        (
            "download-integrity/single-quoted-substitution",
            ".no-mistakes.yml",
        ),
        ("timeout-literals/multiline-containment", ".no-mistakes.yml"),
        ("timeout-literals/multiline-suppressed", ".no-mistakes.yml"),
        ("timeout-literals/symbolic", ".no-mistakes.yml"),
        ("child-links/comment-leading", ".no-mistakes.yml"),
        ("child-links/comment-only", ".no-mistakes.yml"),
        ("child-links/empty-child", ".no-mistakes.yml"),
        ("child-links/inline-suppressed", ".no-mistakes.yml"),
        ("child-links/comment-adjacent", ".no-mistakes.yml"),
        ("child-links/bom-next-line", ".no-mistakes.yml"),
        ("timeout-literals/multiline-allow", ".no-mistakes.yml"),
        ("timeout-literals/multiline-allow-crlf", ".no-mistakes.yml"),
        ("workflow/hash-missing-edge", "config.json"),
        ("workflow/hash-edge-suppressed", "config.json"),
        ("workflow/hash-inventory", "config.json"),
        ("workflow/hash-inventory-suppressed", "config.json"),
        ("workflow/hash-missing-job-yml", "config.json"),
        ("workflow/hash-missing-job-yml-suppressed", "config.json"),
        ("workflow/hash-stale-concurrency", "config.json"),
        (
            "timeout-literals/symbolic-then-unrelated",
            ".no-mistakes.yml",
        ),
        ("timeout-literals/symbolic-then-timeout", ".no-mistakes.yml"),
        ("timeout-literals/multiline-yaml-text", ".no-mistakes.yml"),
        (
            "timeout-literals/single-line-expect-literal",
            ".no-mistakes.yml",
        ),
        ("storybook/required-props-malformed", ".no-mistakes.yml"),
    ]
    .into_iter()
    .map(|(name, config)| {
        let root = base.join(name);
        (name, root.clone(), root.join(config))
    })
    .collect::<Vec<(&str, PathBuf, PathBuf)>>();
    crate::check_json_snapshot::assert_saved_fixture("rule_recall_audit", &cases);
}
