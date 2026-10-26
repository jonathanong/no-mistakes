use super::{push, push_list, push_opt, Field, Kind, Ref};
use crate::config::v2::NoMistakesConfig;

pub(super) fn collect(config: &NoMistakesConfig, refs: &mut Vec<Ref>, origins: bool) {
    let tests = Field::new("tests", origins);
    let playwright = &config.tests.playwright;
    let field = tests.key("playwright");
    push_list(refs, field.key("configs"), Kind::File, &playwright.configs);
    for (index, root) in playwright.selector_roots.iter().enumerate() {
        push(
            refs,
            field.key("selectorRoots").index(index),
            Kind::Directory,
            root,
        );
    }
    push_opt(
        refs,
        field.key("frontendRoot"),
        Kind::Directory,
        playwright.frontend_root.as_deref(),
    );
    push_list(
        refs,
        tests.key("vitest").key("configs"),
        Kind::File,
        &config.tests.vitest.configs,
    );
    push_list(
        refs,
        tests.key("jest").key("configs"),
        Kind::File,
        &config.tests.jest.configs,
    );
    push_list(
        refs,
        tests.key("storybook").key("configs"),
        Kind::File,
        &config.tests.storybook.configs,
    );
    for (index, package) in config.tests.swift.packages.iter().enumerate() {
        push(
            refs,
            tests.key("swift").key("packages").index(index),
            Kind::Directory,
            package,
        );
    }
}
