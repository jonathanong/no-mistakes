use super::*;

/// Applies `spec`'s optional `stripPrefix`/`excludePrefix` transform to a raw
/// extraction, kind-agnostically, same as `minSize`. `stripPrefix` drops any
/// value that does not carry the prefix and strips it from the ones that do;
/// `excludePrefix` then drops any (possibly already-stripped) value that
/// still carries its prefix. Both are no-ops when left empty (the default).
pub(super) fn apply_prefix_transform(spec: &SetSpec, extracted: &mut ExtractedSet) {
    if spec.strip_prefix.is_empty() && spec.exclude_prefix.is_empty() {
        return;
    }
    extracted.values = std::mem::take(&mut extracted.values)
        .into_iter()
        .filter_map(|value| {
            if spec.strip_prefix.is_empty() {
                Some(value)
            } else {
                value
                    .strip_prefix(spec.strip_prefix.as_str())
                    .map(str::to_string)
            }
        })
        .filter(|value| {
            spec.exclude_prefix.is_empty() || !value.starts_with(spec.exclude_prefix.as_str())
        })
        .collect();
}
