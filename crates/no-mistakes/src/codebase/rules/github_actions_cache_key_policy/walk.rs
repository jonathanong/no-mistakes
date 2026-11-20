use super::edit::{self, Slot};
use super::flow::{self, comment_value, list_item, mapping_key, split_kv};
use super::scan::{Job, Step};

#[expect(
    clippy::struct_excessive_bools,
    reason = "independent YAML cursor flags"
)]
pub(super) struct Walker {
    pub(super) action: bool,
    pub(super) in_jobs: bool,
    pub(super) job_open: bool,
    pub(super) jobs_indent: usize,
    pub(super) job_indent: Option<usize>,
    pub(super) in_steps: bool,
    pub(super) steps_indent: usize,
    pub(super) step_dash: Option<usize>,
    pub(super) in_with: bool,
    pub(super) with_indent: usize,
    pub(super) jobs: Vec<Job>,
    pub(super) scalar: Option<edit::Scalar>,
    pub(super) seq: Option<(edit::Seq, usize)>,
}

pub(super) fn walk_jobs(source: &str, action: bool) -> Vec<Job> {
    let mut walker = Walker::new(action);
    for (index, line) in source.lines().enumerate() {
        walker.feed(index + 1, line);
    }
    edit::finish_scalar(&mut walker);
    walker.jobs
}

impl Walker {
    fn new(action: bool) -> Self {
        Self {
            action,
            in_jobs: false,
            job_open: false,
            jobs_indent: 0,
            job_indent: None,
            in_steps: false,
            steps_indent: 0,
            step_dash: None,
            in_with: false,
            with_indent: 0,
            jobs: Vec::new(),
            scalar: None,
            seq: None,
        }
    }

    fn feed(&mut self, line_no: usize, raw: &str) {
        if edit::consume_scalar(self, raw) {
            return;
        }
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "---" || trimmed == "..." {
            return;
        }
        if edit::consume_seq(self, line_no, raw) {
            return;
        }
        if let Some((indent, rest)) = list_item(raw) {
            self.item(line_no, indent, rest);
        } else if let Some((indent, key, value)) = mapping_key(raw) {
            self.key(line_no, indent, key, comment_value(value));
        }
    }

    fn item(&mut self, line_no: usize, indent: usize, rest: &str) {
        if !self.in_steps
            || indent < self.steps_indent
            || self.step_dash.is_some_and(|dash| indent > dash)
        {
            return;
        }
        edit::finish_step(self);
        self.step_dash = Some(indent);
        self.ensure_job();
        self.jobs
            .last_mut()
            .unwrap()
            .steps
            .push(Step::blank(line_no));
        let rest = rest.trim();
        if rest.starts_with('{') {
            if let Some(step) = self.jobs.last_mut().and_then(|job| job.steps.last_mut()) {
                flow::fill_flow(step, line_no, rest);
            }
            return;
        }
        if let Some((key, value)) = split_kv(rest) {
            self.apply_step(line_no, indent, key, comment_value(value));
        }
    }

    fn key(&mut self, line_no: usize, indent: usize, key: &str, value: &str) {
        if !self.action && key == "jobs" && value.is_empty() && !self.in_steps {
            self.in_jobs = true;
            self.jobs_indent = indent;
            return;
        }
        if key == "steps" && value.is_empty() && self.steps_allowed() {
            self.enter_steps(indent);
            return;
        }
        if self.in_steps && self.at_step(line_no, indent, key, value) {
            return;
        }
        if self.in_steps && indent <= self.steps_indent {
            edit::finish_step(self);
            self.in_steps = false;
            self.step_dash = None;
        }
        if !self.action
            && self.in_jobs
            && !self.in_steps
            && value.is_empty()
            && indent > self.jobs_indent
            && self.job_indent.is_none_or(|job| indent == job)
        {
            self.job_indent = Some(indent);
            self.start_job();
        }
    }

    fn at_step(&mut self, line_no: usize, indent: usize, key: &str, value: &str) -> bool {
        let Some(dash) = self.step_dash else {
            return false;
        };
        if self.in_with && indent > self.with_indent {
            self.apply_with(line_no, indent, key, value);
            return true;
        }
        if indent > dash {
            self.in_with = false;
            self.apply_step(line_no, indent, key, value);
            return true;
        }
        false
    }

    fn apply_step(&mut self, line: usize, indent: usize, key: &str, value: &str) {
        match key {
            "id" => edit::assign(self, Slot::Id, line, indent, value),
            "uses" => edit::assign(self, Slot::Uses, line, indent, value),
            "run" => edit::assign(self, Slot::Run, line, indent, value),
            "with" => edit::open_with(self, line, indent, value),
            _ if flow::block_header(value) => edit::assign(self, Slot::Skip, line, indent, value),
            _ => {}
        }
    }

    fn apply_with(&mut self, line: usize, indent: usize, key: &str, value: &str) {
        match key {
            "path" => edit::assign(self, Slot::Path, line, indent, value),
            "key" => edit::assign(self, Slot::Key, line, indent, value),
            "restore-keys" => edit::assign(self, Slot::Restore, line, indent, value),
            _ if flow::block_header(value) => edit::assign(self, Slot::Skip, line, indent, value),
            _ => {}
        }
    }

    fn steps_allowed(&self) -> bool {
        self.action || self.job_open
    }

    fn enter_steps(&mut self, indent: usize) {
        edit::finish_step(self);
        if self.action && self.jobs.is_empty() {
            self.job_open = true;
            self.jobs.push(Job { steps: Vec::new() });
        }
        self.in_steps = true;
        self.steps_indent = indent;
        self.step_dash = None;
    }

    fn start_job(&mut self) {
        edit::finish_step(self);
        self.in_steps = false;
        self.job_open = true;
        self.jobs.push(Job { steps: Vec::new() });
    }

    fn ensure_job(&mut self) {
        if self.jobs.is_empty() {
            self.start_job();
        }
    }
}
