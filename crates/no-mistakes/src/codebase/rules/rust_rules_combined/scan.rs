use super::*;

pub(super) fn scan_file(
    root: &Path,
    path: &Path,
    work: &RustWork,
    exclusive: bool,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    if exclusive {
        let Ok(content) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return scan_file_with_source(root, path, work, &content);
    }
    let Some(content) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    scan_file_with_source(root, path, work, &content)
}

pub(super) fn scan_file_with_source(
    root: &Path,
    path: &Path,
    work: &RustWork,
    content: &str,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for limit in &work.max_limits {
        if let Some(finding) = rust_max_lines_per_file::check_source(path, root, content, *limit) {
            findings.push(finding);
        }
    }

    let inline_tests_enabled =
        work.inline_tests && !has_disable_file_comment(content, RUST_NO_INLINE_TESTS);
    let inline_allows_enabled =
        work.inline_allows && !has_disable_file_comment(content, RUST_NO_INLINE_ALLOWS);
    let needs_inline_tests_parse =
        inline_tests_enabled && content.contains("cfg") && content.contains("test");
    let needs_inline_allows_parse = inline_allows_enabled && content.contains("allow");
    if needs_inline_tests_parse || needs_inline_allows_parse {
        if let Ok(parsed) = syn::parse_file(content) {
            if needs_inline_tests_parse {
                findings.extend(rust_no_inline_tests::findings_from_parsed(
                    path, root, &parsed,
                ));
            }
            if needs_inline_allows_parse {
                findings.extend(rust_no_inline_allows::findings_from_parsed(
                    path, root, &parsed,
                ));
            }
        }
    }

    let mut findings = dedup_findings(findings);
    super::super::suppress_rule_findings_with_source(&mut findings, content);
    findings
}

fn dedup_findings(mut findings: Vec<RuleFinding>) -> Vec<RuleFinding> {
    findings.sort();
    findings.dedup();
    findings
}
