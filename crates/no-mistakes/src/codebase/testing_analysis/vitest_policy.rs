use crate::codebase::check_outcome::CheckFailure;
use no_mistakes_rule_kernel::finding::RuleFinding;

pub const RULE_ID: &str = "vitest-project-policy";

#[derive(Default)]
pub struct Diagnostics {
    pub findings: Vec<RuleFinding>,
    pub incomplete: Vec<CheckFailure>,
}
