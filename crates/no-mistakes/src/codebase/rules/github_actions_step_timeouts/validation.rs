use super::options::family_of;
use super::{Options, GITHUB_MAX_MINUTES, RULE_ID};
use crate::codebase::rules::validate_options::checks::{invalid, option};
use anyhow::Result;
use std::collections::BTreeSet;

impl Options {
    pub(in crate::codebase::rules) fn validate_minutes(&self) -> Result<()> {
        if self.max_minutes == Some(0) {
            return Err(invalid(RULE_ID, option("maxMinutes"), "must be positive"));
        }
        for (index, entry) in self.allow.iter().enumerate() {
            if entry.max_minutes.is_some_and(|minutes| !(1..=GITHUB_MAX_MINUTES).contains(&minutes)) {
                return Err(invalid(RULE_ID, option("allow").index(index).key("maxMinutes"),
                    format_args!("must be in 1..={GITHUB_MAX_MINUTES}")));
            }
        }
        self.validate_action_caps()
    }

    fn validate_action_caps(&self) -> Result<()> {
        let mut seen = BTreeSet::new();
        for (index, entry) in self.action_caps.iter().enumerate() {
            let Some(family) = family_of(&entry.uses) else {
                return Err(invalid(
                    RULE_ID,
                    option("actionCaps").index(index).key("uses"),
                    "must name a concrete action family",
                ));
            };
            if !(1..=GITHUB_MAX_MINUTES).contains(&entry.max_minutes) {
                return Err(invalid(
                    RULE_ID,
                    option("actionCaps").index(index).key("maxMinutes"),
                    format_args!("must be in 1..={GITHUB_MAX_MINUTES}"),
                ));
            }
            if !seen.insert(family) {
                return Err(invalid(
                    RULE_ID,
                    option("actionCaps").index(index),
                    "duplicates an action family",
                ));
            }
        }
        if self.only_configured_actions && self.action_caps.is_empty() {
            return Err(invalid(
                RULE_ID,
                option("onlyConfiguredActions"),
                "requires at least one entry in options.actionCaps",
            ));
        }
        Ok(())
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests {
    use super::Options;

    #[test]
    fn action_caps_fail_before_analysis() {
        for yaml in [
            "onlyConfiguredActions: true",
            "actionCaps: [{uses: actions/setup-node, maxMinutes: 361}]",
            "actionCaps: [{uses: 'actions/*', maxMinutes: 5}]",
        ] {
            let opts: Options = serde_yaml::from_str(yaml).unwrap();
            assert!(opts.validate_minutes().is_err(), "{yaml}");
        }
    }
}
