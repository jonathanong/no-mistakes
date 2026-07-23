use super::*;

pub(in crate::integration_tests) fn load_vitest_json_projects(
    input: ConfigProjectInput<'_>,
) -> Result<Vec<ConfigProject>> {
    let ConfigProjectInput {
        root,
        raw,
        path,
        source,
        config_dir,
        resolver,
        ..
    } = input;
    let parsed =
        test_config::vitest::parse_json_with_resolver(source, path, config_dir, root, resolver)?;
    Ok(parsed
        .into_iter()
        .map(|mut project| {
            project.config = Some(raw.to_string());
            project.workspace = true;
            project
        })
        .collect())
}
