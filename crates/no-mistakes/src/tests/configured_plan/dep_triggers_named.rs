use super::{
    compile_ordered_patterns, ignored_changed_test, matches_ordered,
    structured_trigger_skips_changed_test, DependencyTriggers,
};
use crate::tests::plan::relative_path;
use anyhow::Result;
use no_mistakes::config::v2::schema::NamedFullSuiteTrigger;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn apply_named_triggers(
    root: &Path,
    triggers: &[NamedFullSuiteTrigger],
    changed_files: &[PathBuf],
    discovered_test_files: &HashSet<PathBuf>,
    ignored_sets: &[HashSet<PathBuf>],
    result: &mut DependencyTriggers,
) -> Result<Option<(String, PathBuf)>> {
    let mut legacy_match = None;
    for trigger in triggers {
        let compiled_patterns = compile_ordered_patterns(&trigger.paths)?;
        for changed in changed_files {
            let rel = relative_path(root, changed);
            if !matches_ordered(&compiled_patterns, &rel) {
                continue;
            }
            if trigger.targets.is_empty() {
                if ignored_changed_test(changed, ignored_sets) {
                    continue;
                }
                legacy_match.get_or_insert_with(|| {
                    (
                        format!("{} trigger changed: {}", trigger.name, rel),
                        changed.clone(),
                    )
                });
            } else {
                if structured_trigger_skips_changed_test(
                    changed,
                    discovered_test_files,
                    ignored_sets,
                    trigger.include_changed_tests,
                ) {
                    continue;
                }
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
