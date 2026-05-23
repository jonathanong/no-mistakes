use super::callee::is_candidate_url;

pub fn normalize_url_pattern(url: &str) -> Option<String> {
    if is_candidate_url(url) && !url.contains('*') {
        Some(url.to_string())
    } else {
        glob_url_sample(url)
    }
}

pub fn glob_url_sample(glob: &str) -> Option<String> {
    if !glob.contains('*') {
        return None;
    }

    let (without_scheme, was_leading_wildcard) = glob
        .strip_prefix("**/")
        .map(|value| (value, true))
        .or_else(|| glob.strip_prefix("*/").map(|value| (value, true)))
        .unwrap_or((glob, false));
    let candidate = if is_candidate_url(glob) {
        glob.to_string()
    } else if was_leading_wildcard {
        format!("/{}", without_scheme.trim_start_matches('/'))
    } else if let Some(first_slash) = without_scheme.find('/') {
        let first_segment = &without_scheme[..first_slash];
        if first_segment.contains('.') {
            format!(
                "/{}",
                without_scheme[first_slash + 1..].trim_start_matches('/')
            )
        } else {
            format!("/{}", without_scheme.trim_start_matches('/'))
        }
    } else {
        format!("/{without_scheme}")
    };
    if candidate == "/" || candidate.contains("${") {
        return None;
    }

    let mut sample = String::new();
    let mut chars = candidate.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '*' {
            while matches!(chars.peek(), Some('*')) {
                chars.next();
            }
            sample.push('x');
        } else {
            sample.push(ch);
        }
    }

    is_candidate_url(&sample).then_some(sample)
}
