use super::fixture;
use crate::codebase::ts_source::{
    matching_disable_directive, DisableCommentIndex, DisableDirective,
};

#[test]
fn indexed_matching_conforms_to_the_directive_parser() {
    let cases: &[(&str, &[&str], &[u32])] = &[
        (
            "fixtures/rules/disable-directive-audit/boundaries-crlf.rs",
            &["real-crlf"],
            &[],
        ),
        (
            "fixtures/rules/disable-directive-audit/rust-block-docs-crlf.rs",
            &["bom-inner", "crlf-outer"],
            &[],
        ),
        (
            "fixtures/rules/disable-directive-audit/bom.rs",
            &["rust-max-lines-per-file"],
            &[],
        ),
        (
            "fixtures/check/unused-disable-directives/web/app/stale.ts",
            &[
                "nextjs-no-caching",
                "nextjs-no-caching: obsolete exception",
                "nextjs-no-api-routes",
                "nextjs-no-cache",
                "nextjs-no-cacheing",
            ],
            &[],
        ),
        (
            "fixtures/check/unused-disable-directives/web/app/line.ts",
            &[
                "nextjs-no-caching",
                "nextjs-no-caching: used",
                "unknown-in-string",
                "unknown-in-template",
                "unknown-in-block",
            ],
            &[],
        ),
        (
            "test-cases/rules/doc-consistency/fixture/patterns/docs/apostrophe-disabled.md",
            &["doc-consistency", "doc-consistency "],
            &[2, 3],
        ),
        (
            "test-cases/rules/doc-consistency/fixture/patterns/docs/apostrophe-tracked.mdx",
            &["doc-consistency", "doc-consistency: don't"],
            &[3],
        ),
        (
            "test-cases/rule-recall-audit/fixture/child-links/bom-next-line/child.md",
            &["markdown-child-links", "markdown-child-links: audit"],
            &[],
        ),
    ];
    for &(path, rules, prose_only_lines) in cases {
        let source = std::fs::read_to_string(fixture(path)).unwrap();
        let index = DisableCommentIndex::new(source.as_str());
        let last_line = u32::try_from(source.lines().count()).unwrap();
        for rule in rules
            .iter()
            .copied()
            .chain(["", "missing-rule", "missing rule"])
        {
            for line in std::iter::once(None)
                .chain((0..=last_line + 1).map(Some))
                .chain((0..=last_line + 1).rev().map(Some))
            {
                let quoted = matching_disable_directive(&source, line, rule);
                // The public matcher tracks quotes. The saved apostrophe cases
                // pin the exact provenance where prose mode differs.
                let prose = line
                    .filter(|line| rule == "doc-consistency" && prose_only_lines.contains(line))
                    .map(|line| DisableDirective::Line { line })
                    .or(quoted);
                for (track_quotes, expected) in [(true, quoted), (false, prose)] {
                    assert_eq!(
                        index.matching(rule, line, track_quotes, None),
                        expected,
                        "{path}, rule {rule:?}, line {line:?}, track_quotes={track_quotes}",
                    );
                }
            }
        }
    }
}
