use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::swift::{
    diff_resolved_pins, parse_resolved_pins, SwiftResolvedDiagnostic,
};
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub(crate) struct SwiftResolvedAnalysis {
    pub(crate) seeds: Vec<(PathBuf, PathBuf)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) fallback_triggered: bool,
    handled_files: HashSet<PathBuf>,
}

impl SwiftResolvedAnalysis {
    pub(crate) fn handles(&self, path: &Path) -> bool {
        self.handled_files.contains(path)
    }
}

pub(crate) fn analyze_swift_resolved_changes(
    root: &Path,
    config: &NoMistakesConfig,
    changed_files: &[PathBuf],
    revisions: &RevisionSources,
) -> SwiftResolvedAnalysis {
    let mut analysis = SwiftResolvedAnalysis::default();
    for package in &config.tests.swift.packages {
        let package_root = root.join(package.trim_end_matches('/'));
        let resolved = package_root.join("Package.resolved");
        if !changed_files.contains(&resolved) {
            continue;
        }
        analysis.handled_files.insert(resolved.clone());
        let relative = relative_path(root, &resolved);
        let Some(base) = revisions.base_name() else {
            push_warning(
                &mut analysis,
                "swift-resolved-no-baseline",
                format!(
                    "Could not determine old content of `{relative}`. Provide `--base` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."
                ),
                relative,
            );
            continue;
        };
        let Some(before) = revisions.read_base(&resolved) else {
            push_warning(
                &mut analysis,
                "swift-resolved-no-baseline",
                format!(
                    "Could not read `{relative}` at base ref `{base}`; full-suite selection requires global fallback opt-in"
                ),
                relative,
            );
            continue;
        };
        if revisions.is_diff_only() {
            push_warning(
                &mut analysis,
                "swift-resolved-no-baseline",
                format!(
                    "Could not determine new content of `{relative}` in diff-only mode. Provide `--head` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."
                ),
                relative,
            );
            continue;
        }
        let after = revisions.read_after_or_empty(&resolved);
        let before = match parse_resolved_pins(&before) {
            Ok(pins) => pins,
            Err(diagnostic) => {
                push_diagnostic(&mut analysis, diagnostic, &relative, "base");
                continue;
            }
        };
        let after = match parse_resolved_pins(&after) {
            Ok(pins) => pins,
            Err(diagnostic) => {
                push_diagnostic(&mut analysis, diagnostic, &relative, "head");
                continue;
            }
        };
        if !diff_resolved_pins(&before, &after).is_empty() {
            analysis
                .seeds
                .push((resolved, package_root.join("Package.swift")));
        }
    }
    analysis
}

fn push_diagnostic(
    analysis: &mut SwiftResolvedAnalysis,
    diagnostic: SwiftResolvedDiagnostic,
    file: &str,
    revision: &str,
) {
    let (r#type, description) = match diagnostic {
        SwiftResolvedDiagnostic::Malformed => ("swift-resolved-malformed", "malformed"),
        SwiftResolvedDiagnostic::UnsupportedSchema => (
            "swift-resolved-unsupported-schema",
            "uses an unsupported schema",
        ),
    };
    push_warning(
        analysis,
        r#type,
        format!(
            "`{file}` at {revision} is {description}; full-suite selection requires global fallback opt-in"
        ),
        file.to_string(),
    );
}

fn push_warning(analysis: &mut SwiftResolvedAnalysis, r#type: &str, message: String, file: String) {
    analysis.warnings.push(Warning {
        r#type: r#type.to_string(),
        message,
        file,
        line: None,
    });
    analysis.fallback_triggered = true;
}

#[cfg(test)]
#[path = "swift_resolved_changes_tests.rs"]
mod tests;
