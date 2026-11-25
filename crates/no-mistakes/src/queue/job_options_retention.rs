use super::super::extract_model::{
    FileFacts, JobOptionFinding, JobOptionSite, RetentionValue, UnboundedRetentionPolicy,
};
use super::super::source::relative_string;
use super::defaults::{resolved_default_queue_name, DefaultsIndex};
use crate::config::v2::schema::JobRetentionKey;
use globset::GlobSet;
use std::path::{Path, PathBuf};

const CALL_OPTIONS_OPEN: &str =
    "unbounded-job-retention was not judged for call options because a required retention value is not statically known";
const DEFAULTS_OPEN: &str =
    "unbounded-job-retention was not judged for missing options because defaultJobOptions is not statically known";

#[derive(Default)]
pub(super) struct SiteJudgment {
    pub finding: Option<RetentionNote>,
    pub abstentions: [Option<&'static str>; 2],
}

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
) -> SiteJudgment {
    let unresolved = site.binding.is_some() && site.queue_name.is_none();
    if is_allowed(root, path, allow) || (unresolved && !defaults.has_unresolved_queue(path, site)) {
        return SiteJudgment::default();
    }
    let mut explicit_unbounded = Vec::new();
    let mut unset = Vec::new();
    let mut unknown_call_options = false;
    let mut unknown_defaults = false;
    for &key in &policy.require {
        match (site.evidence.retention.get(key), site.evidence.open) {
            (Some(RetentionValue::Unbounded), _) => explicit_unbounded.push(key),
            (Some(RetentionValue::Bounded), _) => {}
            (Some(RetentionValue::Unknown), _) | (None, true) => unknown_call_options = true,
            (None, false) => {
                let evidence = if site.ambiguous_defaults {
                    None
                } else {
                    defaults.lookup_evidence(site).and_then(Result::ok)
                };
                match evidence.map(|def| (def.retention.get(key), def.open)) {
                    Some((Some(RetentionValue::Bounded | RetentionValue::Unbounded), _)) => {}
                    Some((None, false)) => unset.push(key),
                    _ => unknown_defaults = true,
                }
            }
        }
    }
    let finding = (!(explicit_unbounded.is_empty() && unset.is_empty())).then(|| RetentionNote {
        path: path.to_path_buf(),
        finding: JobOptionFinding {
            kind: "unbounded-job-retention".to_string(),
            line: site.line,
            queue_file: site.queue_file.clone(),
            queue_name: site.queue_name.clone(),
            job: site.job.clone(),
            message: format_producer_message(&explicit_unbounded, &unset),
        },
    });
    SiteJudgment {
        finding,
        abstentions: [
            unknown_call_options.then_some(CALL_OPTIONS_OPEN),
            unknown_defaults.then_some(DEFAULTS_OPEN),
        ],
    }
}

pub(super) fn covered_keys(
    root: &Path,
    path: &Path,
    policy: Option<&UnboundedRetentionPolicy>,
    allow: Option<&GlobSet>,
) -> Vec<&'static str> {
    let Some(policy) = policy.filter(|policy| policy.is_enabled()) else {
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
