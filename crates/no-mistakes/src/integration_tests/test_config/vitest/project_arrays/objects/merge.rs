use crate::integration_tests::test_config::vitest::Options;

pub(super) fn merge_options(base: &mut Options, next: Options) {
    if next.name.is_some() {
        base.name = next.name;
    }
    if next.root.is_some() {
        base.root = next.root;
    }
    if next.include.is_some() {
        base.include = next.include;
    }
    if next.exclude.is_some() {
        base.exclude = next.exclude;
    }
    if next.setup_files.is_some() {
        base.setup_files = next.setup_files;
    }
    if next.global_setup.is_some() {
        base.global_setup = next.global_setup;
    }
    if next.extends.is_some() {
        base.extends = next.extends;
    }
}
