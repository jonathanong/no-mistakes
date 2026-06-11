fn binding_identifier_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

fn concat_candidates(left: &[String], right: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for a in left {
        for b in right {
            out.push(format!("{a}{b}"));
            if out.len() >= 16 {
                return dedupe_candidates(out);
            }
        }
    }
    dedupe_candidates(out)
}

fn normalize_helper_patterns(patterns: Vec<String>) -> Vec<String> {
    let normalized = patterns
        .into_iter()
        .filter_map(|pattern| {
            let pattern = pattern.trim();
            if !pattern.starts_with('/') || should_skip(pattern) {
                return None;
            }
            Some(normalize_next_pathname_pattern(pattern))
        })
        .collect();
    dedupe_candidates(normalized)
}

fn dedupe_candidates(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values.truncate(16);
    values
}
