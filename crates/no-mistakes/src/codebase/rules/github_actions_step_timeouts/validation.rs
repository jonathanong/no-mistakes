use super::{Options, GITHUB_MAX_MINUTES, RULE_ID};
use crate::codebase::rules::validate_options::checks::{invalid, option};
use anyhow::Result;

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
        Ok(())
    }
}
