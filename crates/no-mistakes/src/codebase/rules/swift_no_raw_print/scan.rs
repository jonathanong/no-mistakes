use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
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
        super::super::lexical_mask::swift_code_mask,
        |source, code, rel| {
            let mut seen = BTreeSet::new();
            let mut findings = Vec::new();
            for mat in opts.print.find_iter(code) {
                let print_start = mat.start()
                    + mat
                        .as_str()
                        .find("print")
                        .expect("print regex match contains print");
                let line = source[..print_start]
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
                    message: opts.message.clone(),
                    import: None,
                    target: None,
                });
            }
            findings
        },
    )
}
