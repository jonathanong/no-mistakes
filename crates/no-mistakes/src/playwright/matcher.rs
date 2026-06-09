/// Returns true if `reference` matches `defined_pattern`.
///
/// Reference preprocessing:
/// - strip query and fragment,
/// - strip trailing slash unless the reference is `/`.
///
/// Pattern segments beginning with `:` match one segment. A final `*` segment
/// matches one or more segments. A final `**` segment matches zero or more
/// segments.
pub fn matches(reference: &str, defined_pattern: &str) -> bool {
    let ref_segs = reference_segments(reference);
    let def_segs = pattern_segments(defined_pattern);
    matches_segments(&ref_segs, &def_segs)
}

pub fn reference_segments(reference: &str) -> Vec<&str> {
    let ref_path = reference
        .split('?')
        .next()
        .unwrap_or(reference)
        .split('#')
        .next()
        .unwrap_or(reference);

    let ref_path = if ref_path.len() > 1 && ref_path.ends_with('/') {
        &ref_path[..ref_path.len() - 1]
    } else {
        ref_path
    };

    segments(ref_path)
}

pub fn pattern_segments(pattern: &str) -> Vec<&str> {
    segments(pattern)
}

pub fn matches_segments<S: AsRef<str>>(reference: &[&str], defined_pattern: &[S]) -> bool {
    for (index, def_seg) in defined_pattern.iter().enumerate() {
        let def_seg = def_seg.as_ref();
        let is_last = index + 1 == defined_pattern.len();
        // Catch-all (`**`) and trailing wildcard (`*`) accept any non-empty reference
        // segments, including unresolved interpolations like `${slug}` — each interpolation
        // counts as one concrete segment occupying the wildcard tail.
        if def_seg == "**" && is_last {
            return reference[index..].iter().all(|segment| !segment.is_empty());
        }

        if def_seg == "*" && is_last {
            return reference.len() > index
                && reference[index..].iter().all(|segment| !segment.is_empty());
        }

        let Some(ref_seg) = reference.get(index) else {
            return false;
        };
        if !segment_matches(ref_seg, def_seg) {
            return false;
        }
    }

    reference.len() == defined_pattern.len()
}

fn segments(path: &str) -> Vec<&str> {
    if path == "/" || path.is_empty() {
        Vec::new()
    } else {
        path.strip_prefix('/').unwrap_or(path).split('/').collect()
    }
}

fn segment_matches(reference: &str, defined_pattern: &str) -> bool {
    if reference.is_empty() {
        return false;
    }
    // An unresolved template interpolation (e.g. `${userId}`) stands in for "any single
    // value", so it matches a dynamic route segment (`:param`/`*`) but never a concrete
    // literal. Keeping this explicit guards the behavior if interpolation handling ever
    // changes shape; do not fold it into the literal-equality branch below.
    if segment_is_interpolation(reference) {
        return defined_pattern.starts_with(':') || defined_pattern == "*";
    }
    if defined_pattern.starts_with(':') || defined_pattern == "*" {
        return true;
    }
    reference == defined_pattern
}

/// An unresolved navigation segment that still contains a template interpolation
/// (`${...}`) — for example `${userId}` from `` `/users/${userId}` `` or the synthesized
/// tail of `'/users/' + userId`.
fn segment_is_interpolation(segment: &str) -> bool {
    segment.contains("${")
}

#[cfg(test)]
mod tests;
