use super::RULE_ID;
use crate::codebase::rules::RuleFinding;

pub(super) fn inherited(rel: &str, line: usize, job: &str, grants: &[String]) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: rel.to_owned(),
        line,
        message: format!(
            "{rel}:{line}: job `{job}` inherits workflow-level write permissions ({}) without its own permissions. Move the write scopes to jobs that need them or declare explicit job permissions; inherited write access also flows to reusable workflows.",
            grants.join(", ")
        ),
        import: None,
        target: Some(job.to_owned()),
    }
}

pub(super) fn unresolved(rel: &str, line: usize, job: &str, reason: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: rel.to_owned(),
        line,
        message: format!(
            "{rel}:{line}: cannot determine job permissions ({reason}). Use literal permission values and a mapping of jobs so inherited write access can be checked."
        ),
        import: None,
        target: Some(job.to_owned()),
    }
}
