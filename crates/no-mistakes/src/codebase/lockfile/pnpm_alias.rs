/// Package names in pnpm alias specifiers must not be workspace ranges or paths.
pub(crate) fn valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.starts_with('/')
        && !name.starts_with('*')
        && !name.starts_with('^')
        && !name.starts_with('~')
        && !name.starts_with('<')
        && !name.starts_with('>')
        && !name.starts_with('=')
        && !looks_like_semver_range(name)
        && (!name.starts_with(|c: char| c.is_ascii_digit())
            || name.chars().any(|c| c.is_ascii_alphabetic()))
}

pub(crate) fn looks_like_semver_range(name: &str) -> bool {
    let core = name.split_once(['-', '+']).map_or(name, |(core, _)| core);
    let mut parts = core.split('.');
    let major = parts.next().unwrap_or("");
    let Some(minor) = parts.next() else {
        return false;
    };
    !major.is_empty()
        && !minor.is_empty()
        && major.chars().all(|c| c.is_ascii_digit())
        && semver_range_part(minor)
        && parts.all(semver_range_part)
}

fn semver_range_part(part: &str) -> bool {
    !part.is_empty()
        && (part.chars().all(|c| c.is_ascii_digit())
            || matches!(part.to_ascii_lowercase().as_str(), "x" | "*"))
}
