use no_mistakes::config::v2::NoMistakesConfig;

pub(super) fn mark(config: &mut NoMistakesConfig, handles: impl Fn(&str) -> bool) {
    // Config diagnostics use authored rule indices, so handled slots must stay in place.
    for rule in &mut config.rules {
        if handles(&rule.rule) {
            rule.enabled = false;
        }
    }
}

#[cfg(test)]
#[path = "handled_tests.rs"]
mod tests;
