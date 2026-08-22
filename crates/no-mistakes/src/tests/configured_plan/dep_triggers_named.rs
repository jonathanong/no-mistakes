use super::{compile_ordered_patterns, matches_ordered, DependencyTriggers};
use crate::tests::plan::relative_path;
use anyhow::Result;
use no_mistakes::config::v2::schema::NamedFullSuiteTrigger;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn apply_named_triggers(
    root: &Path,
    triggers: &[NamedFullSuiteTrigger],
    changed_files: &[PathBuf],
    ignored_sets: &[HashSet<PathBuf>],
    result: &mut DependencyTriggers,
) -> Result<Option<(String, PathBuf)>> {
    let mut legacy_match = None;
    for trigger in triggers {
        let compiled_patterns = compile_ordered_patterns(&trigger.paths)?;
        for changed in changed_files {
            if ignored_sets.iter().any(|set| set.contains(changed)) {
                continue;
            }
            let rel = relative_path(root, changed);
            if !matches_ordered(&compiled_patterns, &rel) {
                continue;
            }
            if trigger.targets.is_empty() {
                legacy_match.get_or_insert_with(|| {
                    (
                        format!("{} trigger changed: {}", trigger.name, rel),
                        changed.clone(),
                    )
                });
            } else {
                result
                    .targeted
                    .entry(changed.clone())
                    .or_default()
                    .extend(trigger.targets.iter().cloned());
            }
        }
    }
    Ok(legacy_match)
}

#[cfg(test)]
mod tests {
    use super::{compile_ordered_patterns, matches_ordered};

    #[test]
    fn ordered_patterns_trim_before_detecting_negation() {
        let patterns =
            compile_ordered_patterns(&["src/**".to_string(), " !./src/generated/**".to_string()])
                .unwrap();
        assert!(matches_ordered(&patterns, "src/keep.ts"));
        assert!(!matches_ordered(&patterns, "src/generated/a.ts"));
    }
}
