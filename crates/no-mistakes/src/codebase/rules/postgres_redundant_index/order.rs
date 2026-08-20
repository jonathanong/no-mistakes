use std::cmp::Ordering;

pub(super) fn cmp_sql_rel(left: &str, right: &str) -> Ordering {
    let (left_dir, left_name) = split_rel(left);
    let (right_dir, right_name) = split_rel(right);
    left_dir
        .cmp(right_dir)
        .then_with(|| cmp_versioned_name(left_name, right_name))
}

fn split_rel(rel: &str) -> (&str, &str) {
    rel.rsplit_once('/').unwrap_or(("", rel))
}

fn cmp_versioned_name(left: &str, right: &str) -> Ordering {
    match (version_prefix(left), version_prefix(right)) {
        (Some(left_version), Some(right_version)) => left_version
            .cmp(&right_version)
            .then_with(|| left.cmp(right)),
        _ => left.cmp(right),
    }
}

fn version_prefix(name: &str) -> Option<u64> {
    let start = name.find(|character: char| character.is_ascii_digit())?;
    let digits = name[start..].bytes().take_while(u8::is_ascii_digit).count();
    name[start..start + digits].parse().ok()
}
