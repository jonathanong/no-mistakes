use super::*;
use crate::codebase::ts_source::FileInventory;
use crate::diagnostics::InvocationObserver;
use std::{path::PathBuf, sync::Arc};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn finding(rule: &str, file: &str, line: usize) -> RuleFinding {
    RuleFinding {
        rule: rule.to_owned(),
        file: file.to_owned(),
        line,
        message: format!("violation on line {line}"),
        import: None,
        target: None,
        causes: Vec::new(),
    }
}

fn account(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &SourceStore,
) -> Vec<SuppressedFinding> {
    suppress_domain_findings_with_sources(root, findings, sources, |finding| SuppressionTarget {
        domain: "rules",
        rule: &finding.rule,
        file: &finding.file,
        line: Some(finding.line),
        reason: &finding.message,
        identity: None,
    })
}

#[test]
fn filtering_and_accounting_share_one_scan_per_benchmark_source() {
    for (mode, count, hidden, line_builds, scanned_lines) in [
        ("dense", 3_000, 200, 1, 3_105),
        ("no-directives", 3_000, 0, 1, 0),
        ("single", 1, 1, 1, 3_005),
        ("early-quoted-marker", 1, 0, 1, 6),
        ("file-disable", 3_000, 3_000, 0, 0),
    ] {
        let root = fixture("fixtures/performance/suppression-index").join(mode);
        let file = "src/worker.ts";
        let path = root.join(file);
        let observer = InvocationObserver::new(true);
        let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
        let sources = SourceStore::new_observed(inventory, Some(Arc::clone(&observer)));
        let rule = "forbidden-calls";
        let source = sources.read_path(&path).unwrap();
        let mut filtered = source
            .lines()
            .enumerate()
            .filter(|(_, line)| line.starts_with("scheduler.register("))
            .map(|(index, _)| finding(rule, file, index + 1))
            .collect::<Vec<_>>();
        assert_eq!(filtered.len(), count, "{mode}");

        let mut accounted = filtered.clone();
        let (_, suppressed) = rayon::join(
            || suppress_rule_findings_with_sources(&root, &mut filtered, &sources),
            || account(&root, &mut accounted, &sources),
        );
        assert_eq!(filtered, accounted, "{mode}");
        assert_eq!(filtered.len(), count - hidden, "{mode}");
        assert_eq!(suppressed.len(), hidden, "{mode}");
        for item in suppressed {
            let line = item.line.unwrap();
            let expected = match mode {
                "file-disable" => (SuppressionDirectiveKind::File, 1),
                "dense"
                    if source
                        .lines()
                        .nth(line - 1)
                        .unwrap()
                        .ends_with("benchmark exception") =>
                {
                    (SuppressionDirectiveKind::Line, line)
                }
                _ if mode == "dense" => (SuppressionDirectiveKind::NextLine, line - 1),
                _ => (SuppressionDirectiveKind::Line, line),
            };
            assert_eq!((item.directive.kind, item.directive.line), expected);
            assert_eq!(item.source_file, file);
            assert_eq!(item.reason, format!("violation on line {line}"));
        }

        let work = observer.snapshot().work;
        let count = |name| work.get(name).copied().unwrap_or_default();
        assert_eq!(count("suppression.header_builds"), 1, "{mode}");
        assert_eq!(
            count("suppression.line_index_builds"),
            line_builds,
            "{mode}"
        );
        assert_eq!(count("suppression.line_visits"), scanned_lines, "{mode}");
        assert_eq!(
            count("suppression.lexer_line_visits"),
            scanned_lines,
            "{mode}"
        );
        assert_eq!(count("source.reads"), 1, "{mode}");
    }
}

#[test]
fn markdown_prose_and_mdx_keep_their_quote_modes() {
    let root = fixture("test-cases/rules/doc-consistency/fixture/patterns");
    let markdown = "docs/apostrophe-disabled.md";
    let mdx = "docs/apostrophe-tracked.mdx";
    let rule = "doc-consistency";
    let sources =
        crate::codebase::rules::source_store_for_files(&[root.join(markdown), root.join(mdx)]);
    let mut findings = vec![
        finding(rule, markdown, 2),
        finding(rule, markdown, 3),
        finding(rule, mdx, 1),
        finding(rule, mdx, 3),
        finding(rule, mdx, 5),
    ];
    let suppressed = account(&root, &mut findings, &sources);

    assert_eq!(findings, [finding(rule, mdx, 3)]);
    for (item, expected) in suppressed.iter().zip([
        (markdown, 2, SuppressionDirectiveKind::Line, 2),
        (markdown, 3, SuppressionDirectiveKind::Line, 3),
        (mdx, 1, SuppressionDirectiveKind::Line, 1),
        (mdx, 5, SuppressionDirectiveKind::NextLine, 4),
    ]) {
        assert_eq!(
            (
                item.file.as_str(),
                item.line.unwrap(),
                item.directive.kind,
                item.directive.line
            ),
            expected,
        );
    }
    assert_eq!(suppressed.len(), 4);
}

#[test]
fn file_directive_wins_over_both_line_directives() {
    let root = fixture("fixtures/check/unused-disable-directives");
    let file = "web/app/stale.ts";
    let sources = crate::codebase::rules::source_store_for_files(&[root.join(file)]);
    let mut findings = vec![
        finding("nextjs-no-caching", file, 3),
        finding("nextjs-no-caching", file, 4),
    ];
    let suppressed = account(&root, &mut findings, &sources);

    assert!(findings.is_empty());
    assert_eq!(suppressed.len(), 2);
    assert!(suppressed.iter().all(|item| {
        item.directive
            == SuppressionDirective {
                kind: SuppressionDirectiveKind::File,
                line: 1,
            }
    }));
}

#[path = "index_conformance_tests.rs"]
mod conformance;

#[path = "index_session_tests.rs"]
mod session;
