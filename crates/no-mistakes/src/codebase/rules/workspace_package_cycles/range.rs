#[derive(Clone, Copy, PartialEq, Eq)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre: bool,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

/// npm/Yarn range match for workspace linking. `*`, `^`, `~`, comparators,
/// `||`, and partial versions (`1`, `1.2`) follow the usual semver meaning.
/// A prerelease satisfies only `*` or its exact version.
pub(super) fn version_satisfies(range: &str, version: &str) -> bool {
    let range = range.trim();
    let Some(actual) = parse_version(version.trim()) else {
        return false;
    };
    if matches!(range, "*" | "x" | "X") {
        return true;
    }
    range
        .split("||")
        .any(|clause| clause_satisfies(clause.trim(), actual))
}

fn clause_satisfies(clause: &str, version: Version) -> bool {
    !clause.is_empty()
        && clause
            .split_whitespace()
            .all(|token| token_satisfies(token, version))
}

fn token_satisfies(token: &str, version: Version) -> bool {
    if matches!(token, "*" | "x" | "X") {
        return true;
    }
    let (operator, rest) = split_operator(token);
    match operator {
        Some("^") => caret_satisfies(rest, version),
        Some("~") => tilde_satisfies(rest, version),
        Some(">=") => ordered(rest, version, std::cmp::Ordering::Less, false),
        Some("<=") => ordered(rest, version, std::cmp::Ordering::Greater, false),
        Some(">") => ordered(rest, version, std::cmp::Ordering::Less, true),
        Some("<") => ordered(rest, version, std::cmp::Ordering::Greater, true),
        Some("=") => exact_satisfies(rest, version),
        None => bare_satisfies(token, version),
        Some(_) => false,
    }
}

fn split_operator(token: &str) -> (Option<&str>, &str) {
    for operator in [">=", "<=", ">", "<", "=", "^", "~"] {
        if let Some(rest) = token.strip_prefix(operator) {
            return (Some(operator), rest);
        }
    }
    (None, token)
}

fn caret_satisfies(base_text: &str, version: Version) -> bool {
    let Some(base) = comparable_base(base_text, version) else {
        return false;
    };
    if version < base {
        return false;
    }
    if base.major > 0 {
        version.major == base.major
    } else if base.minor > 0 {
        version.major == 0 && version.minor == base.minor
    } else {
        version.major == 0 && version.minor == 0 && version.patch == base.patch
    }
}

fn tilde_satisfies(base_text: &str, version: Version) -> bool {
    let Some(base) = comparable_base(base_text, version) else {
        return false;
    };
    if version < base {
        return false;
    }
    let parts = base_text.split('.').count();
    version.major == base.major && (parts <= 1 || version.minor == base.minor)
}

fn ordered(base_text: &str, version: Version, reject: std::cmp::Ordering, strict: bool) -> bool {
    let Some(base) = comparable_base(base_text, version) else {
        return false;
    };
    let ordering = version.cmp(&base);
    ordering != reject && (!strict || ordering != std::cmp::Ordering::Equal)
}

fn exact_satisfies(base_text: &str, version: Version) -> bool {
    comparable_base(base_text, version)
        .is_some_and(|base| version == base && version.pre == base.pre)
}

fn bare_satisfies(token: &str, version: Version) -> bool {
    if token.chars().any(|ch| matches!(ch, '*' | 'x' | 'X')) {
        return wildcard_satisfies(token, version);
    }
    let Some(base) = comparable_base(token, version) else {
        return false;
    };
    if version < base {
        return false;
    }
    match token.split('.').count() {
        1 => version.major == base.major,
        2 => version.major == base.major && version.minor == base.minor,
        _ => version == base,
    }
}

fn wildcard_satisfies(token: &str, version: Version) -> bool {
    if version.pre {
        return false;
    }
    let numbers = [version.major, version.minor, version.patch];
    for (index, part) in token.split('.').enumerate() {
        if matches!(part, "*" | "x" | "X") {
            continue;
        }
        let Some(parsed) = part.parse::<u64>().ok() else {
            return false;
        };
        if numbers.get(index).copied() != Some(parsed) {
            return false;
        }
    }
    true
}

fn comparable_base(text: &str, version: Version) -> Option<Version> {
    let base = parse_version(text)?;
    (version.pre == base.pre).then_some(base)
}

fn parse_version(input: &str) -> Option<Version> {
    let input = input.split_once('+').map(|(core, _)| core).unwrap_or(input);
    let (core, pre) = match input.split_once('-') {
        Some((core, pre)) if !pre.is_empty() => (core, true),
        _ => (input, false),
    };
    let mut numbers = Vec::with_capacity(3);
    for part in core.split('.') {
        numbers.push(part.parse::<u64>().ok()?);
    }
    if numbers.is_empty() || numbers.len() > 3 {
        return None;
    }
    while numbers.len() < 3 {
        numbers.push(0);
    }
    Some(Version {
        major: numbers[0],
        minor: numbers[1],
        patch: numbers[2],
        pre,
    })
}
