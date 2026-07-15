pub(super) fn extract_test_target_names(package_swift: &str) -> Vec<String> {
    let mut names: Vec<String> = super::parse_manifest_targets(package_swift)
        .into_iter()
        .filter(|target| target.is_test)
        .map(|target| target.name)
        .collect();
    names.sort();
    names.dedup();
    names
}
