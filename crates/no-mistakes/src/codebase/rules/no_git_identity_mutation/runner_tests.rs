use super::runner::is_managed_runner_only;

#[test]
fn is_managed_runner_only_inline_list_all_managed() {
    let content = "runs-on: [ubuntu-latest, windows-latest]\n";
    assert!(
        is_managed_runner_only(content),
        "bracket list of managed runners should be recognized"
    );
}

#[test]
fn is_managed_runner_only_inline_list_mixed() {
    let content = "runs-on: [ubuntu-latest, self-hosted]\n";
    assert!(
        !is_managed_runner_only(content),
        "bracket list with self-hosted should not be recognized as managed-only"
    );
}

#[test]
fn is_managed_runner_only_trailing_comma_skips_empty_element() {
    // Exercises line 95: `continue` when a split segment is empty (e.g. trailing comma).
    let content = "runs-on: [ubuntu-latest,]\n";
    assert!(
        is_managed_runner_only(content),
        "trailing comma produces empty element that should be skipped, not treated as unmanaged"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_managed() {
    // Exercises EW60S: multi-line YAML list form strips leading "- " before matching.
    let content = "runs-on:\n  - ubuntu-latest\n";
    assert!(
        is_managed_runner_only(content),
        "multi-line YAML list with managed runner should be recognized"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_self_hosted() {
    let content = "runs-on:\n  - self-hosted\n";
    assert!(
        !is_managed_runner_only(content),
        "multi-line YAML list with self-hosted runner should not be recognized as managed-only"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_later_self_hosted() {
    let content = "runs-on:\n  - ubuntu-latest\n  - self-hosted\n";
    assert!(
        !is_managed_runner_only(content),
        "multi-line YAML list must inspect every runner entry"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_skips_blank_and_comment_lines() {
    let content = "runs-on:\n\n  # hosted runners\n  - ubuntu-latest\n";
    assert!(
        is_managed_runner_only(content),
        "blank and comment lines inside runs-on lists should be ignored"
    );
}

#[test]
fn is_managed_runner_only_mapping_labels_managed() {
    let content = "runs-on:\n  group: hosted\n  labels: ubuntu-latest\n";
    assert!(
        is_managed_runner_only(content),
        "mapping labels with a managed runner should be recognized"
    );
}

#[test]
fn is_managed_runner_only_mapping_labels_self_hosted() {
    let content = "runs-on:\n  group: hosted\n  labels: self-hosted\n";
    assert!(
        !is_managed_runner_only(content),
        "mapping labels with self-hosted should not be recognized as managed-only"
    );
}

#[test]
fn is_managed_runner_only_multiline_non_list_stops_collection() {
    let content = "runs-on:\n  name: ubuntu-runners\n  - ubuntu-latest\n";
    assert!(
        !is_managed_runner_only(content),
        "non-list YAML values should not be treated as managed runner entries"
    );
}
