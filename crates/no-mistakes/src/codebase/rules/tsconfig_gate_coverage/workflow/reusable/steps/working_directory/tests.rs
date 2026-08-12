use super::{step_working_directory, working_directory_exists, EnvironmentState, InputState};
use serde_yaml::Value;
use std::collections::BTreeSet;

#[test]
fn dynamic_step_directory_cannot_fall_through_to_a_later_run_step() {
    let step: Value = serde_yaml::from_str("working-directory: '${{ vars.dir }}'").unwrap();
    assert_eq!(
        step_working_directory(
            &step,
            &InputState::new(),
            &EnvironmentState::default(),
            &None,
        ),
        None
    );
}

#[test]
fn working_directories_require_a_tracked_descendant_not_a_file_at_the_same_path() {
    let paths = BTreeSet::from([
        "README.md".to_string(),
        "packages/app/tsconfig.json".to_string(),
    ]);
    assert!(!working_directory_exists("README.md", &paths));
    assert!(working_directory_exists("packages/app", &paths));
    assert!(working_directory_exists(".", &paths));
}
