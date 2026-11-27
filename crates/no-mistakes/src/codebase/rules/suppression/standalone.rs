use super::{note_standalone_fallback, DisableCommentIndex, RuleFinding};
use no_mistakes_rule_kernel::paths::normalize_discovery_path;
use no_mistakes_rule_kernel::suppression::{self, SuppressionSources};
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct StandaloneSuppression {
    canonical_root: Option<PathBuf>,
}

impl SuppressionSources for StandaloneSuppression {
    type Index = DisableCommentIndex<Arc<str>>;

    fn load_index(&self, _: &Path, candidate: &Path, _: bool) -> Option<Self::Index> {
        note_standalone_fallback();
        let canonical_root = self.canonical_root.as_deref()?;
        let path = std::fs::canonicalize(candidate).ok()?;
        let metadata = std::fs::metadata(&path).ok()?;
        if !path.starts_with(canonical_root) || !metadata.is_file() {
            return None;
        }
        let source = Arc::<str>::from(std::fs::read_to_string(candidate).ok()?);
        Some(DisableCommentIndex::new(source))
    }

    const ALLOW_ABSOLUTE_PATHS: bool = false;
}

pub(in crate::codebase::rules) fn suppress_rule_findings_inner(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    already_suppressed_rules: &[&str],
) {
    let sources = StandaloneSuppression {
        canonical_root: std::fs::canonicalize(normalize_discovery_path(root)).ok(),
    };
    suppression::suppress_rule_findings_with_sources_except(
        root,
        findings,
        &sources,
        already_suppressed_rules,
    );
}
