use std::path::PathBuf;

pub fn assert_saved_fixture_clean(name: &str, cases: &[(&str, PathBuf, PathBuf)]) {
    super::check_json_snapshot::assert_saved_fixture_checked(name, cases, &[], true);
}
