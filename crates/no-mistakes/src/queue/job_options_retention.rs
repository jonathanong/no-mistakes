use super::super::extract_model::{
    FileFacts, JobOptionFinding, JobOptionSite, RetentionValue, UnboundedRetentionPolicy,
};
use super::super::source::relative_string;
use super::defaults::{resolved_default_queue_name, DefaultsIndex};
use crate::config::v2::schema::JobRetentionKey;
use globset::GlobSet;
use std::path::{Path, PathBuf};

pub(super) struct RetentionNote {
    pub path: PathBuf,
    pub finding: JobOptionFinding,
}

pub(super) fn judge_file_defaults(
    root: &Path,
    path: &Path,
    file: &FileFacts,
    policy: &UnboundedRetentionPolicy,
    allow: Option<&GlobSet>,
) -> Vec<RetentionNote> {
    if is_allowed(root, path, allow) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for defaults in &file.queue_defaults {
        let queue_name = resolved_default_queue_name(file, defaults);
        let false_keys = policy
            .require
            .iter()
            .copied()
            .filter(|&key| defaults.evidence.retention.get(key) == Some(RetentionValue::Unbounded))
            .collect::<Vec<_>>();
        if false_keys.is_empty() {
            continue;
        }
        out.push(RetentionNote {
            path: path.to_path_buf(),
            finding: JobOptionFinding {
                kind: "unbounded-job-retention".to_string(),
                line: defaults.line,
                queue_file: Some(path.to_path_buf()),
                queue_name,
                job: None,
                message: format!(
                    "defaultJobOptions keeps {} without a bound",
                    format_keys(&false_keys)
                ),
            },
        });
    }
    out
}

pub(super) fn judge_site(
    root: &Path,
    path: &Path,
    site: &JobOptionSite,
    policy: &UnboundedRetentionPolicy,
    allow: Option<&GlobSet>,
    defaults: &DefaultsIndex,
) -> Option<RetentionNote> {
    if is_allowed(root, path, allow) {
        return None;
    }
    if site.binding.is_some() && site.queue_name.is_none() {
        return None;
    }
    let mut explicit_unbounded = Vec::new();
    let mut unset = Vec::new();
    for &key in &policy.require {
        match site.evidence.retention.get(key) {
            Some(RetentionValue::Unbounded) => explicit_unbounded.push(key),
            Some(RetentionValue::Bounded | RetentionValue::Unknown) => {}
            None => {
                if site.evidence.open || site.ambiguous_defaults {
                    continue;
                }
                match defaults.lookup_evidence(site) {
                    Some(Err(())) => {}
                    Some(Ok(def)) => match def.retention.get(key) {
                        Some(
                            RetentionValue::Bounded
                            | RetentionValue::Unknown
                            | RetentionValue::Unbounded,
                        ) => {}
                        None if def.open => {}
                        None => unset.push(key),
                    },
                    None => unset.push(key),
                }
            }
        }
    }
    if explicit_unbounded.is_empty() && unset.is_empty() {
        return None;
    }
    Some(RetentionNote {
        path: path.to_path_buf(),
        finding: JobOptionFinding {
            kind: "unbounded-job-retention".to_string(),
            line: site.line,
            queue_file: site.queue_file.clone(),
            queue_name: site.queue_name.clone(),
            job: site.job.clone(),
            message: format_producer_message(&explicit_unbounded, &unset),
        },
    })
}

pub(super) fn covered_keys(
    root: &Path,
    path: &Path,
    policy: Option<&UnboundedRetentionPolicy>,
    allow: Option<&GlobSet>,
) -> Vec<&'static str> {
    let Some(policy) = policy else {
        return Vec::new();
    };
    if is_allowed(root, path, allow) {
        return Vec::new();
    }
    policy.require.iter().map(|key| key.as_str()).collect()
}

fn is_allowed(root: &Path, path: &Path, allow: Option<&GlobSet>) -> bool {
    allow.is_some_and(|globset| globset.is_match(relative_string(root, path)))
}

fn format_keys(keys: &[JobRetentionKey]) -> String {
    keys.iter()
        .map(|key| key.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_producer_message(
    explicit_unbounded: &[JobRetentionKey],
    unset: &[JobRetentionKey],
) -> String {
    let mut parts = Vec::new();
    if !explicit_unbounded.is_empty() {
        parts.push(format!(
            "{} kept without a bound",
            format_keys(explicit_unbounded)
        ));
    }
    if !unset.is_empty() {
        parts.push(format!("{} unset", format_keys(unset)));
    }
    format!("unbounded job retention: {}", parts.join("; "))
}
