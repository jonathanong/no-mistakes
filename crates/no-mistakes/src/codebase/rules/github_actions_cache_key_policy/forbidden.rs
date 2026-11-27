use super::options::Options;
use super::scan::Step;
use super::text::norm_path;
use super::hit;
use no_mistakes_rule_kernel::finding::RuleFinding;

pub(super) fn check(
    rel: &str,
    step: &Step,
    paths: &[String],
    options: &Options,
    rule_message: Option<&str>,
    findings: &mut Vec<RuleFinding>,
) {
    let line = step
        .path
        .as_ref()
        .map(|field| field.line)
        .unwrap_or(step.line);
    let custom = options
        .forbidden_paths_message
        .as_deref()
        .map(str::trim)
        .or(rule_message);
    for raw in &options.forbidden_paths {
        let path = norm_path(raw);
        if !paths.iter().any(|entry| entry == &path) {
            continue;
        }
        let message = custom.map(str::to_string).unwrap_or_else(|| {
            format!(
                "Caching `{path}` is forbidden by forbiddenPaths. Delete this cache step or remove `{path}` from its path input."
            )
        });
        findings.push(hit(rel, line, message));
    }
}
