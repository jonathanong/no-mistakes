use super::*;

pub(super) fn check_source(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    cond_set: &GlobSet,
    patterns: &[Regex; 3],
    content: &str,
) -> Vec<RuleFinding> {
    let rel_str = relative_slash_path(root, path);
    if exclude_set.is_match(rel_str.as_str()) {
        return Vec::new();
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(ext, "sh" | "bash" | "zsh" | "fish" | "yml" | "yaml")
        && !has_shell_shebang(content)
    {
        return Vec::new();
    }

    if cond_set.is_match(rel_str.as_str()) && is_managed_runner_only(content) {
        return Vec::new();
    }

    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();

    for pattern_re in patterns {
        for captures in pattern_re.captures_iter(content) {
            let mat = captures.get(0).expect("complete match");
            // A global option argument named --get is not a config query flag.
            let config_arguments = captures
                .name("config")
                .map_or(mat.as_str(), |config| &content[config.end()..mat.end()]);
            if is_read_only_config_match(config_arguments) {
                continue;
            }
            if mat
                .as_str()
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_digit())
                && content[mat.end()..].starts_with('>')
            {
                continue;
            }
            let prefix = &content[..mat.start()];
            // When the match starts at '\n', the actual keyword is on the following
            // line; adjust line_start accordingly before checking for comments.
            let starts_at_newline = content.as_bytes().get(mat.start()) == Some(&b'\n');
            let line_start = if starts_at_newline {
                mat.start() + 1
            } else {
                prefix.rfind('\n').map_or(0, |i| i + 1)
            };
            let line_text = content[line_start..]
                .lines()
                .next()
                .unwrap_or("")
                .trim_start();
            if line_text.starts_with('#')
                || line_text.starts_with("echo ")
                || line_text.starts_with("printf ")
            {
                continue;
            }
            let newline_count = prefix.bytes().filter(|&b| b == b'\n').count();
            let line = newline_count + 1 + usize::from(starts_at_newline);
            findings.push(RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: file.clone(),
                line,
                message: format!(
                    "{file}:{line}: git config user.name/email is banned \
                    \u{2014} use GIT_AUTHOR_*/GIT_COMMITTER_* env vars instead"
                ),
                import: None,
                target: None,
            });
        }
    }

    findings
}

fn is_read_only_config_match(matched: &str) -> bool {
    matched
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | '(' | ')' | '[' | ']'))
        .any(|token| {
            let token = token.trim_matches(|c| matches!(c, '\'' | '"' | '`'));
            matches!(
                token,
                "--get" | "--get-all" | "--get-regexp" | "--get-urlmatch"
            )
        })
}
