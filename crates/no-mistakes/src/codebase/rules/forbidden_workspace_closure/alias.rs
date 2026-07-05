pub(super) fn resolved_dependency_name(specifier: &str) -> Option<String> {
    let stripped = specifier
        .strip_prefix("workspace:")
        .or_else(|| specifier.strip_prefix("npm:"))?;
    let aliased = stripped.strip_prefix("npm:").unwrap_or(stripped);
    package_name_from_alias_specifier(aliased)
}

fn package_name_from_alias_specifier(specifier: &str) -> Option<String> {
    if let Some(stripped) = specifier.strip_prefix('@') {
        let slash = stripped.find('/')?;
        let name_start = slash + 2;
        let rest = specifier.get(name_start..)?;
        let version_start = rest.find('@').unwrap_or(rest.len());
        let name = specifier.get(..name_start + version_start)?;
        return valid_package_name(name).then(|| name.to_string());
    }
    let version_start = specifier.find('@').unwrap_or(specifier.len());
    let name = specifier.get(..version_start)?;
    valid_package_name(name).then(|| name.to_string())
}

fn valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.starts_with('/')
        && !name.starts_with('*')
        && !name.starts_with('^')
        && !name.starts_with('~')
}
