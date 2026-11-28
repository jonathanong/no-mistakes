mod cont;
mod cursor;
mod flow;
mod job_line;
mod scalar;

pub(crate) use flow::yaml_plain;
pub(crate) use job_line::job_key_line;

#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub name: String,
    pub line: usize,
    pub value_line: usize,
    pub value: String,
    pub block_key: bool,
}

#[derive(Debug)]
pub(crate) struct Step {
    pub line: usize,
    pub fields: Vec<Field>,
}

impl Step {
    pub(super) fn blank(line: usize) -> Self {
        Self {
            line,
            fields: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Job {
    pub name: String,
    pub steps: Vec<Step>,
}

pub(crate) fn walk_steps(source: &str, action: bool) -> Vec<Job> {
    let mut walker = cursor::Walker::new(action);
    for (index, line) in source.lines().enumerate() {
        walker.feed(index + 1, line);
    }
    scalar::finish_scalar(&mut walker);
    walker.jobs
}

pub(crate) fn first_key_line(source: &str, job_id: &str, index: usize, keys: &[&str]) -> usize {
    let Some(job) = walk_steps(source, job_id == "(composite)")
        .into_iter()
        .find(|job| job.name == job_id)
    else {
        return 1;
    };
    let Some(step) = job.steps.get(index) else {
        return 1;
    };
    keys.iter()
        .find_map(|key| {
            step.fields
                .iter()
                .find(|field| field.block_key && field.name == *key)
                .map(|field| field.line)
        })
        .unwrap_or(step.line)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
