use super::ProducerBinding;
use oxc_ast::ast::IdentifierReference;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct JobOptionSpread {
    pub name: String,
    pub at: u32,
}

impl From<&IdentifierReference<'_>> for JobOptionSpread {
    fn from(id: &IdentifierReference<'_>) -> Self {
        Self {
            name: id.name.to_string(),
            at: id.span.start,
        }
    }
}

/// Keys present on a job-options object, plus spreads resolved at the end of the file.
#[derive(Debug, Clone, Default)]
pub(crate) struct JobOptionEvidence {
    pub keys: BTreeSet<String>,
    pub open: bool,
    pub spreads: Vec<JobOptionSpread>,
    /// When set, finish folds each name in `spreads` as this member, in order.
    pub member: Option<String>,
}

impl JobOptionEvidence {
    pub(crate) fn open() -> Self {
        Self {
            open: true,
            ..Self::default()
        }
    }

    pub(crate) fn absorb(&mut self, other: &Self) {
        self.keys.extend(other.keys.iter().cloned());
        self.spreads.extend(other.spreads.iter().cloned());
        self.open |= other.open;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct JobOptionSite {
    pub line: usize,
    pub binding: Option<ProducerBinding>,
    pub queue_name: Option<String>,
    pub queue_file: Option<PathBuf>,
    pub declaration: Option<u32>,
    pub ambiguous_defaults: bool,
    pub job: Option<String>,
    pub evidence: JobOptionEvidence,
}

#[derive(Debug, Clone)]
pub(crate) struct QueueDefaultOptions {
    pub declaration: u32,
    pub queue_name: Option<String>,
    pub evidence: JobOptionEvidence,
}

#[derive(Debug, Clone)]
pub(crate) struct JobOptionFinding {
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
