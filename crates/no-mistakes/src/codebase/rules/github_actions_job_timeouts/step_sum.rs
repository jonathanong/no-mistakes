use super::{yaml::timeout_minutes, RuleFinding};

pub(super) fn finding(
    rel: &str,
    line: usize,
    job_id: &str,
    job: &serde_yaml::Mapping,
    job_timeout: u64,
) -> Option<RuleFinding> {
    let steps = job.get("steps")?.as_sequence()?;
    if steps.len() < 2 {
        return None;
    }
    let mut sum = 0u128;
    let mut largest = 0u64;
    for step in steps {
        let minutes = timeout_minutes(step.as_mapping()?.get("timeout-minutes")?)?;
        sum += u128::from(minutes);
        largest = largest.max(minutes);
    }
    if u128::from(job_timeout) < sum {
        return None;
    }
    Some(super::yaml::finding(
        rel,
        line,
        format!(
            "{rel}: job \"{job_id}\" timeout-minutes is {job_timeout}, at or above its step timeout sum of {sum} (largest step timeout: {largest}); choose a job timeout below the step sum and high enough for the largest step plus overhead so the job deadline provides a tighter total runtime bound"
        ),
        job_id,
    ))
}
