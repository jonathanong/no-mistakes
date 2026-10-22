use crate::config::v2::NoMistakesConfig;

/// Only file-local Exact checks can judge source call sites without resolving
/// canonical targets. Any malformed or broader selector keeps the old graph.
pub(super) fn eligible(config: &NoMistakesConfig) -> bool {
    let applications = config.rule_applications(super::RULE_ID);
    !applications.is_empty()
        && applications.into_iter().all(|application| {
            let options = &application.options;
            if options.get("traversal").and_then(serde_yaml::Value::as_str) != Some("file")
                || options
                    .get("unknownCalls")
                    .is_some_and(|value| value.as_str() != Some("ignore"))
            {
                return false;
            }
            let Some(roots) = options
                .get("roots")
                .and_then(serde_yaml::Value::as_sequence)
            else {
                return false;
            };
            let Some(targets) = options
                .get("targets")
                .and_then(serde_yaml::Value::as_sequence)
            else {
                return false;
            };
            !roots.is_empty()
                && !targets.is_empty()
                && roots.iter().all(|root| {
                    root.as_mapping().is_some_and(|map| map.len() == 1)
                        && root
                            .get("file")
                            .and_then(serde_yaml::Value::as_str)
                            .is_some_and(|file| !file.is_empty())
                })
                && targets.iter().all(|target| {
                    target
                        .get("exact")
                        .and_then(serde_yaml::Value::as_str)
                        .is_some_and(|name| !name.is_empty() && name != "<unknown>")
                })
        })
}
