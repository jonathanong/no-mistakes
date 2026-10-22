use crate::codebase::ts_source::facts::ArgumentProjection;
use crate::config::v2::NoMistakesConfig;

/// Read the same raw rule options used for fact demand. Ambiguous selectors
/// retain full argument facts so option validation keeps its original order.
pub(super) fn projection(config: &NoMistakesConfig) -> Option<ArgumentProjection> {
    let mut exact = Vec::new();
    let mut required = false;
    for application in config.rule_applications(super::RULE_ID) {
        let Some(targets) = application.options.get("targets") else {
            continue;
        };
        let Some(targets) = targets.as_sequence() else {
            // Preserve option parsing as the first error for malformed targets.
            continue;
        };
        for target in targets {
            let Some(condition) = target.get("whenArguments") else {
                continue;
            };
            if condition.is_null() {
                continue;
            }
            required = true;
            let Some(name) = target.get("exact").and_then(serde_yaml::Value::as_str) else {
                return Some(ArgumentProjection::default());
            };
            if name == "<unknown>" {
                return Some(ArgumentProjection::default());
            }
            exact.push(name.to_owned());
        }
    }
    required.then(|| ArgumentProjection::exact(exact))
}
