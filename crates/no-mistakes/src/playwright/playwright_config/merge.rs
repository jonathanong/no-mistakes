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

    let mut test_match = project
        .test_match
        .or_else(|| root.test_match.clone())
        .unwrap_or_else(|| crate::playwright::test_patterns::Patterns {
            globs: default_test_match(),
            ..Default::default()
        });
    let mut test_ignore = root.test_ignore.clone().unwrap_or_default();
    test_ignore.extend(project.test_ignore.unwrap_or_default());
    TestProject {
        name: project.name.or_else(|| root.name.clone()),
        config_dir: config_dir.to_path_buf(),
        test_dir: project
            .test_dir
            .or_else(|| root.test_dir.clone())
            .unwrap_or_else(|| ".".to_string()),
        test_match: std::mem::take(&mut test_match.globs),
        test_ignore: std::mem::take(&mut test_ignore.globs),
        pattern_options: crate::playwright::test_patterns::PatternOptions {
            test_match,
            test_ignore,
        },
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
    path: &Path,
    bindings: &std::collections::BTreeMap<String, &oxc_ast::ast::Expression<'_>>,
) -> anyhow::Result<ParsedOptions> {
    use super::ast_nav::{expression_config_object, property_expression};
    use super::literals::{optional_string, required_string};

    use oxc_span::GetSpan;
    let property_start = |name| {
        object
            .properties
            .iter()
            .find_map(|property| {
                let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                (crate::ast::property_key_name(&property.key).as_deref() == Some(name))
                    .then_some(property.span().start)
            })
            .unwrap_or(object.span().start)
    };
    let use_object = property_expression(object, "use").and_then(|value| {
        let mut seen = std::collections::BTreeSet::new();
        expression_config_object(value, bindings, &mut seen)
    });

    Ok(ParsedOptions {
        name: property_expression(object, "name").and_then(|value| optional_string(value, source)),
        test_dir: property_expression(object, "testDir")
            .map(|value| required_string(value, source, "testDir"))
            .transpose()?,
        test_match: property_expression(object, "testMatch").map(|value| {
            crate::playwright::test_patterns::Patterns::parse(
                value,
                source,
                path,
                "testMatch",
                property_start("testMatch"),
            )
        }),
        test_ignore: property_expression(object, "testIgnore").map(|value| {
            crate::playwright::test_patterns::Patterns::parse(
                value,
                source,
                path,
                "testIgnore",
                property_start("testIgnore"),
            )
        }),
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

pub(super) fn default_test_match() -> Vec<String> {
    DEFAULT_TEST_MATCH
        .iter()
        .map(|pattern| pattern.to_string())
        .collect()
}

pub(super) fn default_config(root: &Path) -> super::types::PlaywrightConfig {
    super::types::PlaywrightConfig {
        name: None,
        projects: vec![super::types::TestProject {
            name: None,
            config_dir: root.to_path_buf(),
            test_dir: ".".to_string(),
            test_match: default_test_match(),
            test_ignore: Vec::new(),
            pattern_options: Default::default(),
            base_url: None,
            // Synthesized fallback config: the attribute was not read from a real
            // Playwright config, so leave it `None` to defer to `selectors.testIds`.
            test_id_attribute: None,
        }],
    }
}

pub(super) fn missing_config_name_error(name: &str) -> anyhow::Error {
    anyhow::Error::msg(format!("no Playwright config found with name {name}"))
}

pub(super) fn validate_config_names(
    configs: &[(std::path::PathBuf, super::types::PlaywrightConfig)],
    config_name_filter: Option<&str>,
) -> anyhow::Result<()> {
    if configs.len() <= 1 && config_name_filter.is_none() {
        return Ok(());
    }

    let mut seen = std::collections::BTreeMap::new();
    for (path, config) in configs {
        let Some(name) = config.name.as_deref() else {
            anyhow::bail!(
                "Playwright config {} must define top-level name when multiple configs are analyzed or --project is used",
                path.display()
            );
        };
        if let Some(previous) = seen.insert(name.to_string(), path.display().to_string()) {
            anyhow::bail!(
                "Playwright config name {name} is duplicated by {} and {}",
                previous,
                path.display()
            );
        }
    }
    Ok(())
}
