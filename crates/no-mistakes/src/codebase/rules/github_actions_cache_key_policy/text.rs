pub(super) fn is_cache_action(uses: &str) -> bool {
    let name = uses.split('@').next().unwrap_or(uses).trim();
    let name = name.trim_matches(['"', '\'']).trim_end_matches('/');
    matches!(
        name.to_ascii_lowercase().as_str(),
        "actions/cache" | "actions/cache/restore" | "actions/cache/save"
    )
}

pub(super) fn yaml_plain(value: &str) -> String {
    super::super::github_actions_workflow_files::step_lines::yaml_plain(value)
}

pub(super) fn norm_path(path: &str) -> String {
    let text = yaml_plain(path);
    let text = text.trim();
    let text = text.strip_prefix("./").unwrap_or(text);
    text.trim_end_matches('/').to_string()
}

pub(super) fn path_entries(value: &str) -> Vec<String> {
    value
        .lines()
        .map(norm_path)
        .filter(|path| !path.is_empty())
        .collect()
}

pub(super) fn has_fragment(value: &str, fragment: &str) -> bool {
    let fragment = fragment.trim();
    expressions(value)
        .iter()
        .any(|body| contains_token(&strip_expr_strings(body), fragment, false))
}

pub(super) fn command_hashes_lockfile(cmd: &str, lockfile: &str) -> bool {
    super::shell_cmd::parts(cmd)
        .iter()
        .any(|part| part_hashes(part, lockfile))
}

fn part_hashes(part: &str, lockfile: &str) -> bool {
    const TOOLS: &[&str] = &[
        "sha256sum",
        "sha384sum",
        "sha512sum",
        "sha224sum",
        "sha1sum",
        "shasum",
        "b2sum",
        "b3sum",
        "md5sum",
        "cksum",
    ];
    let tool = TOOLS.iter().any(|tool| contains_token(part, tool, false))
        || (contains_token(part, "openssl", false) && contains_token(part, "dgst", false));
    tool && mentions_path(part, lockfile)
}

pub(super) fn hash_args(value: &str) -> Vec<String> {
    let mut args = Vec::new();
    for body in expressions(value) {
        push_hash_args(&body, &mut args);
    }
    args
}

pub(super) fn mentions_path(run: &str, lockfile: &str) -> bool {
    let code: String = run.lines().map(shell_line).collect::<Vec<_>>().join("\n");
    contains_token(&code, &norm_path(lockfile), true)
}

fn expressions(value: &str) -> Vec<String> {
    let mut bodies = Vec::new();
    let mut rest = value;
    while let Some(start) = rest.find("${{") {
        rest = &rest[start + 3..];
        let Some(end) = rest.find("}}") else { break };
        bodies.push(rest[..end].trim().to_string());
        rest = &rest[end + 2..];
    }
    bodies
}

fn contains_token(hay: &str, token: &str, path: bool) -> bool {
    if token.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(rel) = hay[start..].find(token) {
        let at = start + rel;
        let before_ok = at == 0 || !continues(prev_char(&hay[..at]), path);
        let after = at + token.len();
        let after_ok = after >= hay.len() || !continues(next_char(&hay[after..]), path);
        if before_ok && after_ok {
            return true;
        }
        start = at + 1;
    }
    false
}

fn continues(c: char, path: bool) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || (path && c == '-')
}

fn prev_char(text: &str) -> char {
    text.chars().next_back().unwrap_or(' ')
}

fn next_char(text: &str) -> char {
    text.chars().next().unwrap_or(' ')
}

pub(super) fn shell_line(line: &str) -> &str {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return "";
    }
    line.split(" #").next().unwrap_or(line)
}

fn strip_expr_strings(body: &str) -> String {
    let mut out = String::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\'' && c != '"' {
            out.push(c);
            continue;
        }
        let quote = c;
        while let Some(next) = chars.next() {
            if quote == '"' && next == '\\' {
                chars.next();
                continue;
            }
            if next == quote {
                if quote == '\'' && chars.peek() == Some(&'\'') {
                    chars.next();
                    continue;
                }
                break;
            }
        }
        out.push(' ');
    }
    out
}

fn push_hash_args(body: &str, args: &mut Vec<String>) {
    let mut rest = body;
    while let Some(rel) = rest.find("hashFiles") {
        let before_ok = rel == 0 || !continues(prev_char(&rest[..rel]), false);
        rest = &rest[rel + "hashFiles".len()..];
        if !before_ok {
            continue;
        }
        let trimmed = rest.trim_start();
        if !trimmed.starts_with('(') {
            continue;
        }
        let inner = &trimmed[1..];
        let Some(end) = inner.find(')') else { continue };
        for part in inner[..end].split(',') {
            let part = yaml_plain(part);
            if !part.is_empty() {
                args.push(part);
            }
        }
        rest = &inner[end + 1..];
    }
}
