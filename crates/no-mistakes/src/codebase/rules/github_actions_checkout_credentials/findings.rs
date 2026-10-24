use super::{RuleFinding, RULE_ID};

pub(super) fn uncertain(
    rel: &str,
    line: usize,
    job: &str,
    step: &str,
    reason: &str,
) -> RuleFinding {
    let message = format!(
        "{rel}:{line}: job `{job}` step {step}: cannot analyze checkout credentials ({reason}). \
         Set with.persist-credentials to literal false, or use the default GITHUB_TOKEN with \
         literal read-only permissions; persisted write credentials let later steps modify \
         GitHub state."
    );
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message,
        import: None,
        target: Some(job.to_string()),
    }
}

pub(super) fn finding(
    rel: &str,
    line: usize,
    job: &str,
    step: usize,
    reason: &str,
    declared_maximum: bool,
) -> RuleFinding {
    let access = if declared_maximum {
        "the reusable workflow's declared permissions may allow GITHUB_TOKEN write access"
    } else {
        "the GITHUB_TOKEN can write"
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: format!(
            "{rel}:{line}: job `{job}` step {step} uses actions/checkout while {access} \
             ({reason}), but persist-credentials is not false. actions/checkout writes \
             that token into the git credential store unless with.persist-credentials is false."
        ),
        import: None,
        target: Some(job.to_string()),
    }
}

pub(super) fn reference(
    rel: &str,
    line: usize,
    job: &str,
    step: usize,
    known: bool,
) -> RuleFinding {
    let reason = if known {
        "checks out a pull-request head ref in a trusted pull_request_target workflow"
    } else {
        "cannot analyze the checkout ref or its repository in a pull_request_target workflow"
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: format!(
            "{rel}:{line}: job `{job}` step {step} {reason}. Use the base repository \
             (omit with.repository or use github.repository) and a statically known base-side \
             ref (omit with.ref or use github.sha), AND set with.persist-credentials to literal \
             false. Pull-request head code may expose trusted credentials and runner context \
             even when credential persistence is disabled."
        ),
        import: None,
        target: Some(job.to_string()),
    }
}
