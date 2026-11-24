use super::ProducerBinding;
use crate::queue::job_options::RetentionEvidence;
use oxc_ast::ast::IdentifierReference;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct JobOptionSpread {
    pub name: String,
    pub at: u32,
    after: RetentionEvidence,
}

impl From<&IdentifierReference<'_>> for JobOptionSpread {
    fn from(id: &IdentifierReference<'_>) -> Self {
        Self {
            name: id.name.to_string(),
            at: id.span.start,
            after: RetentionEvidence::default(),
        }
    }
}

/// Keys present on a job-options object, plus spreads resolved at the end of the file.
#[derive(Debug, Clone, Default)]
pub(crate) struct JobOptionEvidence {
    pub keys: BTreeSet<String>,
    pub open: bool,
    pub spreads: Vec<JobOptionSpread>,
    pub(in crate::queue) retention: RetentionEvidence,
    /// Initial options when the pending spreads supply the whole `opts` member.
    pub before_opts_spreads: Option<Box<Self>>,
}

impl JobOptionEvidence {
    pub(crate) fn open() -> Self {
        let mut evidence = Self::default();
        evidence.mark_open();
        evidence
    }

    pub(crate) fn mark_open(&mut self) {
        self.open = true;
        *self.retention_tail() = RetentionEvidence::UNKNOWN;
    }

    pub(in crate::queue) fn retention_tail(&mut self) -> &mut RetentionEvidence {
        self.spreads
            .last_mut()
            .map_or(&mut self.retention, |spread| &mut spread.after)
    }

    pub(crate) fn absorb(&mut self, other: &Self) {
        self.keys.extend(other.keys.iter().cloned());
        self.open |= other.open;
        self.retention_tail().overlay(other.retention);
        self.spreads.extend(other.spreads.iter().cloned());
    }

    pub(crate) fn resolve_spreads(&mut self, mut lookup: impl FnMut(&JobOptionSpread) -> Self) {
        for spread in std::mem::take(&mut self.spreads) {
            self.absorb(&lookup(&spread));
            self.retention.overlay(spread.after);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct JobOptionSite {
    pub line: usize,
    pub binding: Option<ProducerBinding>,
    pub queue_name: Option<String>,
    pub queue_file: Option<PathBuf>,
    pub job: Option<String>,
    pub evidence: JobOptionEvidence,
}

#[derive(Debug, Clone)]
pub(crate) struct QueueDefaultOptions {
    pub declaration: u32,
    pub line: usize,
    pub queue_name: Option<String>,
    pub evidence: JobOptionEvidence,
}

#[derive(Debug, Clone)]
pub(crate) struct JobOptionFinding {
    pub kind: &'static str,
    pub line: usize,
    pub queue_file: Option<PathBuf>,
    pub queue_name: Option<String>,
    pub job: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RequiredJobOptions {
    pub global: Vec<String>,
    pub by_queue: BTreeMap<String, Vec<String>>,
}

impl RequiredJobOptions {
    pub(crate) fn for_queue(&self, name: Option<&str>) -> &[String] {
        name.and_then(|name| self.by_queue.get(name))
            .map(Vec::as_slice)
            .unwrap_or(&self.global)
    }
}
