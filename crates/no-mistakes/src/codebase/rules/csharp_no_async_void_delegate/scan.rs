use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    super::super::scan_masked_file(
        super::super::MaskedFileContext::new(
            root,
            path,
            RULE_ID,
            &opts.allow,
            sources,
            defer_suppression,
        ),
        super::super::lexical_mask::csharp_code_mask,
        |_, code, rel| {
            let mut seen = BTreeSet::new();
            let mut findings = Vec::new();
            for pattern in opts.constructors.iter().chain(opts.methods.iter()) {
                collect_matches(code, pattern, rel, &opts.message, &mut seen, &mut findings);
            }
            findings
        },
    )
}

fn collect_matches(
    source: &str,
    pattern: &Regex,
    rel: &str,
    message: &str,
    seen: &mut BTreeSet<usize>,
    findings: &mut Vec<RuleFinding>,
) {
    for mat in pattern.find_iter(source) {
        let line = source[..mat.start()]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;
        if !seen.insert(line) {
            continue;
        }
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line,
            message: message.to_string(),
            import: None,
            target: None,
        });
    }
}
