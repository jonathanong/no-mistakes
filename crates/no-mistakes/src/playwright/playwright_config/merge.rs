use super::types::{ParsedOptions, TestProject};
use std::path::Path;

pub const DEFAULT_TEST_MATCH: &[&str] = &[
    "**/*.spec.ts",
    "**/*.spec.tsx",
    "**/*.spec.js",
    "**/*.spec.jsx",
    "**/*.spec.mts",
    "**/*.spec.cts",
    "**/*.spec.mjs",
    "**/*.spec.cjs",
    "**/*.test.ts",
    "**/*.test.tsx",
    "**/*.test.js",
    "**/*.test.jsx",
    "**/*.test.mts",
    "**/*.test.cts",
    "**/*.test.mjs",
    "**/*.test.cjs",
];
pub const DEFAULT_TEST_ID_ATTRIBUTE: &str = "data-testid";

pub(super) fn merge_project(
    config_dir: &Path,
    root: &ParsedOptions,
    project: Option<ParsedOptions>,
) -> TestProject {
    let project = project.unwrap_or_default();

    TestProject {
        name: project.name.or_else(|| root.name.clone()),
        config_dir: config_dir.to_path_buf(),
        test_dir: project
            .test_dir
            .or_else(|| root.test_dir.clone())
            .unwrap_or_else(|| ".".to_string()),
        test_match: project
            .test_match
            .or_else(|| root.test_match.clone())
            .unwrap_or_else(default_test_match),
        test_ignore: combine(root.test_ignore.clone(), project.test_ignore),
        base_url: project.base_url.or_else(|| root.base_url.clone()),
        // Kept as `Option`: `None` means the attribute was not statically
        // readable from the config, which lets coverage fall back to the
        // configured `selectors.testIds`. `DEFAULT_TEST_ID_ATTRIBUTE` is only
        // applied at the final resolution step (see `analysis::context`).
        test_id_attribute: project
            .test_id_attribute
            .or_else(|| root.test_id_attribute.clone()),
    }
}

pub(super) fn parse_options(
    object: &oxc_ast::ast::ObjectExpression<'_>,
    source: &str,
    bindings: &std::collections::BTreeMap<String, &oxc_ast::ast::Expression<'_>>,
) -> anyhow::Result<ParsedOptions> {
    use super::ast_nav::{expression_config_object, property_expression};
    use super::literals::{optional_string, required_string, required_string_or_array};

    let use_object = property_expression(object, "use").and_then(|value| {
        let mut seen = std::collections::BTreeSet::new();
        expression_config_object(value, bindings, &mut seen)
    });

    Ok(ParsedOptions {
        name: property_expression(object, "name").and_then(|value| optional_string(value, source)),
        test_dir: property_expression(object, "testDir")
            .map(|value| required_string(value, source, "testDir"))
            .transpose()?,
        test_match: property_expression(object, "testMatch")
            .map(|value| required_string_or_array(value, source, "testMatch"))
            .transpose()?,
        test_ignore: property_expression(object, "testIgnore")
            .map(|value| required_string_or_array(value, source, "testIgnore"))
            .transpose()?,
        base_url: use_object
            .and_then(|value| property_expression(value, "baseURL"))
            .or_else(|| property_expression(object, "baseURL"))
            .and_then(|value| optional_string(value, source)),
        test_id_attribute: use_object
            .and_then(|value| property_expression(value, "testIdAttribute"))
            .or_else(|| property_expression(object, "testIdAttribute"))
            .and_then(|value| optional_string(value, source)),
    })
}

fn combine(left: Option<Vec<String>>, right: Option<Vec<String>>) -> Vec<String> {
    let mut values = left.unwrap_or_default();
    values.extend(right.unwrap_or_default());
    values
}

pub(super) fn default_test_match() -> Vec<String> {
    DEFAULT_TEST_MATCH
        .iter()
        .map(|pattern| pattern.to_string())
        .collect()
}
