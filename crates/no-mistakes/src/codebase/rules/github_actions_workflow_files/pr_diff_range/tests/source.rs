use super::super::{options::CompiledOptions, runs::check_parsed, RULE_ID};
use super::super::super::gha_kind;
use no_mistakes_rule_kernel::finding::RuleFinding;
use serde_yaml::Value;

pub(super) fn check_source_with_options(
    rel: &str,
    source: &str,
    opts: &CompiledOptions,
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    if gha_kind(rel) != Some(false) {
        return (Vec::new(), Vec::new());
    }
    let source = source.trim_start_matches('\u{feff}');
    let value = match serde_yaml::from_str::<Value>(source) {
        Ok(value) => value,
        Err(error) => {
            let line = error.location().map_or(1, |loc| loc.line());
            let note = RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line,
                message: format!(
                    "cannot verify {RULE_ID}: YAML could not be parsed ({error}); repair source syntax so this check can verify the file"
                ),
                import: None,
                target: Some("unanalyzable".to_string()),
            };
            return (Vec::new(), vec![note]);
        }
    };
    check_parsed(rel, source, &value, opts)
}
