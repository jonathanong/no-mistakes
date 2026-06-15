//! The `infra test-for` query and its helpers, split out to keep each source
//! file within the repository's per-file line budget.

use std::path::Path;

use super::{InfraReport, TestForRow};
use crate::codebase::terraform::TfBlockKind;
use crate::codebase::ts_resolver::normalize_path;

impl InfraReport {
    /// Test files covering the resources defined in `tf_file`, per the configured
    /// test convention.
    pub fn test_for(&self, tf_file: &str) -> Vec<TestForRow> {
        let abs = normalize_path(&self.root.join(tf_file));
        // Only a parsed, configured Terraform file has a module to cover; an
        // unknown or mistyped path must not fall back to every module test.
        let Some(file_facts) = self.facts.files.get(&abs) else {
            return Vec::new();
        };
        let module_dir = file_facts.module_dir.clone();
        let anchor = match &self.test.test_root {
            Some(test_root) => normalize_path(&self.root.join(test_root)),
            None => module_dir,
        };
        let Some(globset) = &self.test_globset else {
            return Vec::new();
        };

        let declared = self.declared_addresses(&abs);
        let match_resource = self.test.match_mode.as_deref() != Some("module");

        let mut rows: Vec<TestForRow> = self
            .files
            .iter()
            .filter(|file| {
                file.strip_prefix(&anchor)
                    .is_ok_and(|rel| globset.is_match(rel))
            })
            .filter(|file| !match_resource || self.test_references_declared(file, &declared))
            .map(|file| TestForRow {
                test_file: self.rel(file),
            })
            .collect();
        rows.sort();
        rows.dedup();
        rows
    }

    fn declared_addresses(&self, tf_file: &Path) -> Vec<String> {
        self.facts
            .files
            .get(tf_file)
            .map(|file| {
                file.blocks
                    .iter()
                    .filter(|block| matches!(block.kind, TfBlockKind::Resource | TfBlockKind::Data))
                    .map(|block| block.addr.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn test_references_declared(&self, file: &Path, declared: &[String]) -> bool {
        if declared.is_empty() {
            return false;
        }
        match std::fs::read_to_string(file) {
            Ok(content) => declared
                .iter()
                .any(|addr| references_address(&content, addr)),
            Err(_) => false,
        }
    }
}

/// Whether `content` references `addr` on identifier boundaries, so that neither
/// `aws_s3_bucket.foo_logs` (trailing) nor `legacy_aws_s3_bucket.foo` (leading)
/// match a file declaring only `aws_s3_bucket.foo`.
fn references_address(content: &str, addr: &str) -> bool {
    content.match_indices(addr).any(|(index, _)| {
        let before = content[..index].chars().next_back();
        let after = content[index + addr.len()..].chars().next();
        !is_identifier_char(before) && !is_identifier_char(after)
    })
}

fn is_identifier_char(ch: Option<char>) -> bool {
    ch.is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
}
