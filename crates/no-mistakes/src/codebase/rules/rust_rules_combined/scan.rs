use super::*;

pub(super) fn scan_file_outputs(
    root: &Path,
    path: &Path,
    work: &RustWork,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
    parse: impl FnOnce(&str) -> syn::Result<syn::File>,
) -> Vec<(Option<usize>, RuleFinding)> {
    let content = match sources.read_path(path) {
        Ok(content) => content,
        Err(error) => {
            let make = |rule| super::super::source_access::read_failure(root, path, rule, &error);
            let mut findings: Vec<_> = work
                .max_limits
                .iter()
                .map(|limit| (Some(*limit), make(RUST_MAX_LINES_PER_FILE)))
                .collect();
            if work.inline_tests {
                findings.push((None, make(RUST_NO_INLINE_TESTS)));
            }
            if work.inline_allows {
                findings.push((None, make(RUST_NO_INLINE_ALLOWS)));
            }
            if !defer_suppression {
                findings.retain(|(_, finding)| {
                    let mut candidate = vec![finding.clone()];
                    super::super::source_access::suppress_invalid_utf8(&mut candidate, &error);
                    !candidate.is_empty()
                });
            }
            return findings;
        }
    };
    scan_outputs_with_parser(root, path, work, &content, defer_suppression, parse)
}

pub(super) fn scan_outputs_with_parser(
    root: &Path,
    path: &Path,
    work: &RustWork,
    content: &str,
    defer_suppression: bool,
    parse: impl FnOnce(&str) -> syn::Result<syn::File>,
) -> Vec<(Option<usize>, RuleFinding)> {
    let mut findings = Vec::new();
    for limit in &work.max_limits {
        if let Some(finding) = rust_max_lines_per_file::check_source_with_deferred_suppression(
            path,
            root,
            content,
            *limit,
            defer_suppression,
        ) {
            findings.push((Some(*limit), finding));
        }
    }

    let inline_tests_enabled = work.inline_tests
        && (defer_suppression || !has_disable_file_comment(content, RUST_NO_INLINE_TESTS));
    let inline_allows_enabled = work.inline_allows
        && (defer_suppression || !has_disable_file_comment(content, RUST_NO_INLINE_ALLOWS));
    let needs_inline_tests_parse =
        inline_tests_enabled && content.contains("cfg") && content.contains("test");
    let needs_inline_allows_parse = inline_allows_enabled && content.contains("allow");
    if needs_inline_tests_parse || needs_inline_allows_parse {
        match parse(content) {
            Ok(parsed) => {
                if needs_inline_tests_parse {
                    findings.extend(
                        rust_no_inline_tests::findings_from_parsed(path, root, &parsed)
                            .into_iter()
                            .map(|finding| (None, finding)),
                    );
                }
                if needs_inline_allows_parse {
                    findings.extend(
                        rust_no_inline_allows::findings_from_parsed(path, root, &parsed)
                            .into_iter()
                            .map(|finding| (None, finding)),
                    );
                }
            }
            Err(error) => {
                for (requested, rule) in [
                    (needs_inline_tests_parse, RUST_NO_INLINE_TESTS),
                    (needs_inline_allows_parse, RUST_NO_INLINE_ALLOWS),
                ] {
                    if requested {
                        findings.push((
                            None,
                            super::super::source_access::unanalyzable_source(
                                root,
                                path,
                                rule,
                                format!("cannot parse Rust: {error}"),
                            ),
                        ));
                    }
                }
            }
        }
    }
    let mut findings: Vec<_> = findings
        .into_iter()
        .filter(|(_, finding)| {
            defer_suppression || {
                let mut candidate = vec![finding.clone()];
                super::super::suppress_rule_findings_with_source(&mut candidate, content);
                !candidate.is_empty()
            }
        })
        .collect();
    findings.sort();
    findings.dedup();
    findings
}
