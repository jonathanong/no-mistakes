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
}
