use super::ProducerBinding;
use crate::config::v2::schema::{JobRetentionKey, UnboundedJobRetentionConfig};
use oxc_ast::ast::IdentifierReference;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetentionValue {
    Bounded,
    Unbounded,
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RetentionEvidence {
    pub remove_on_complete: Option<RetentionValue>,
    pub remove_on_fail: Option<RetentionValue>,
}

impl RetentionEvidence {
    pub(crate) fn get(&self, key: JobRetentionKey) -> Option<RetentionValue> {
        match key {
            JobRetentionKey::RemoveOnComplete => self.remove_on_complete,
            JobRetentionKey::RemoveOnFail => self.remove_on_fail,
        }
    }

    pub(crate) fn set(&mut self, key: JobRetentionKey, value: RetentionValue) {
        match key {
            JobRetentionKey::RemoveOnComplete => self.remove_on_complete = Some(value),
            JobRetentionKey::RemoveOnFail => self.remove_on_fail = Some(value),
        }
    }

    pub(crate) fn unknown_all() -> Self {
        Self {
            remove_on_complete: Some(RetentionValue::Unknown),
            remove_on_fail: Some(RetentionValue::Unknown),
        }
    }

    pub(crate) fn invalidate_unset(&mut self) {
        self.remove_on_complete = Some(RetentionValue::Unknown);
        self.remove_on_fail = Some(RetentionValue::Unknown);
    }

    pub(crate) fn absorb(&mut self, other: &Self, other_open: bool) {
        if other_open {
            if other.remove_on_complete.is_none() {
                self.remove_on_complete = Some(RetentionValue::Unknown);
            }
            if other.remove_on_fail.is_none() {
                self.remove_on_fail = Some(RetentionValue::Unknown);
            }
        }
        if let Some(value) = other.remove_on_complete {
            self.remove_on_complete = Some(value);
        }
        if let Some(value) = other.remove_on_fail {
            self.remove_on_fail = Some(value);
        }
    }
}

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
    pub retention: RetentionEvidence,
}

impl JobOptionEvidence {
    pub(crate) fn open() -> Self {
        Self {
            open: true,
            retention: RetentionEvidence::unknown_all(),
            ..Self::default()
        }
    }

    pub(crate) fn absorb(&mut self, other: &Self) {
        self.keys.extend(other.keys.iter().cloned());
        self.spreads.extend(other.spreads.iter().cloned());
        self.open |= other.open;
        self.retention.absorb(&other.retention, other.open);
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
    pub line: usize,
    pub queue_name: Option<String>,
    pub evidence: JobOptionEvidence,
}

#[derive(Debug, Clone)]
pub(crate) struct JobOptionFinding {
    pub kind: String,
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

pub(crate) type UnboundedRetentionPolicy = UnboundedJobRetentionConfig;
