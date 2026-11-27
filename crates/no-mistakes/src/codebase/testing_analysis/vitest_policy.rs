use crate::codebase::check_outcome::CheckFailure;
use no_mistakes_rule_kernel::finding::RuleFinding;

#[derive(Default)]
pub struct Diagnostics {
    pub findings: Vec<RuleFinding>,
    pub incomplete: Vec<CheckFailure>,
}
