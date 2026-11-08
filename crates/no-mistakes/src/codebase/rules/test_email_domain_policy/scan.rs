use super::*;

pub(super) fn check_file_with_sources(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !opts.extensions.iter().any(|ext| rel.ends_with(ext)) {
        return Vec::new();
    }
    let Some(content) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    if let Some(audit) = &opts.audit {
        audit.checked(path);
    }
    if !opts.domain_needles.is_empty()
        && !content.contains('&')
        && super::super::byte_prefilter::is_ascii(content.as_bytes())
        && !super::super::byte_prefilter::contains_any_ascii_ignore_case(
            content.as_bytes(),
            &opts.domain_needles,
        )
    {
        PREFILTER_SKIPS.with(|skips| skips.set(skips.get() + 1));
        return Vec::new();
    }
    scan_email_lines(&rel, &content, opts)
}

pub(super) fn scan_email_lines(
    rel: &str,
    content: &str,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let decoded = encoding::decode(line);
        for found in opts.email.find_iter(&decoded.text) {
            let matched = decoded.original(line, found.range());
            let domain = email_domain(found.as_str());
            if !opts.banned_domains.contains(&domain) {
                continue;
            }
            let matches = opts.allowed.iter().map(|allowed| allowed.is_match(matched));
            let suppressed = if let Some(audit) = &opts.audit {
                audit.audit.usage.suppress(matches)
            } else {
                matches.into_iter().any(|matched| matched)
            };
            if suppressed {
                continue;
            }
            findings.push(RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line: index + 1,
                message: message(rel, &domain, opts.replacement.as_deref()),
                import: Some(matched.to_string()),
                target: Some(domain),
            });
        }
    }
    findings
}
