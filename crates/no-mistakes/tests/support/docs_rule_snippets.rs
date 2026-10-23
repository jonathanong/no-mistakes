use std::path::{Path, PathBuf};

use tempfile::TempDir;

const SKIP_OPEN_PRS: &[&str] = &[
    // Open PR #2425 owns these check-rule implementations.
    "postgres-no-mutable-sql-registry",
    "postgres-no-unbounded-sql-identifier",
];

pub struct RulePage {
    pub file: String,
    pub body: String,
}

pub fn skip_open_pr(rule_id: &str) -> bool {
    SKIP_OPEN_PRS.contains(&rule_id)
}

pub fn labeled_section(body: &str, labels: &[&str]) -> String {
    for label in labels {
        let headed = heading_section(body, label);
        if !headed.trim().is_empty() {
            return headed;
        }
        let bold = format!("**{label}:**");
        if let Some(start) = body.find(&bold) {
            let rest = &body[start + bold.len()..];
            let end = rest.find("\n**").unwrap_or(rest.len());
            return rest[..end].to_string();
        }
    }
    String::new()
}

pub fn heading_section(body: &str, title: &str) -> String {
    let header = format!("## {title}").to_ascii_lowercase();
    let mut in_fence: Option<(u8, usize)> = None;
    let mut collecting = false;
    let mut collected = Vec::new();
    for line in body.lines() {
        if let Some(mark) = in_fence {
            if close_fence(line, mark) {
                in_fence = None;
            }
            if collecting {
                collected.push(line);
            }
            continue;
        }
        if let Some((mark, _)) = open_fence(line) {
            in_fence = Some(mark);
            if collecting {
                collected.push(line);
            }
            continue;
        }
        if collecting {
            if line.starts_with("## ") {
                break;
            }
            collected.push(line);
            continue;
        }
        if heading_matches(line, &header) {
            collecting = true;
        }
    }
    collected.join("\n")
}

fn heading_matches(line: &str, header_lc: &str) -> bool {
    let line_lc = line.to_ascii_lowercase();
    line_lc == header_lc
        || line_lc.starts_with(&format!("{header_lc} and "))
        || line_lc.starts_with(&format!("{header_lc},"))
}

fn open_fence(line: &str) -> Option<((u8, usize), String)> {
    let trimmed = line.trim_start();
    let character = *trimmed.as_bytes().first()?;
    if !matches!(character, b'`' | b'~') {
        return None;
    }
    let mark = trimmed.bytes().take_while(|&c| c == character).count();
    if mark < 3 {
        return None;
    }
    let rest = trimmed[mark..].trim();
    let lang = rest
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    Some(((character, mark), lang))
}

fn close_fence(line: &str, (character, mark): (u8, usize)) -> bool {
    let trimmed = line.trim_start();
    let n = trimmed.bytes().take_while(|&c| c == character).count();
    n >= mark && trimmed.chars().skip(n).all(char::is_whitespace)
}

pub fn counterexample_section(body: &str) -> String {
    let headed = heading_section(body, "Counterexample");
    if !headed.trim().is_empty() {
        return headed;
    }
    let valid = heading_section(body, "Valid example");
    if let Some(idx) = valid.to_ascii_lowercase().find("counterexample") {
        return valid[idx..].to_string();
    }
    String::new()
}

pub fn fences(section: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut lines = section.lines().peekable();
    while let Some(line) = lines.next() {
        let Some((mark, lang)) = open_fence(line) else {
            continue;
        };
        let mut code = Vec::new();
        for inner in lines.by_ref() {
            if close_fence(inner, mark) {
                break;
            }
            code.push(inner);
        }
        let body = code.join("\n");
        if !body.trim().is_empty() {
            out.push((lang, body));
        }
    }
    out
}

pub fn rule_config_yaml(body: &str, rule_id: &str) -> String {
    let needle = format!("rule: {rule_id}");
    for (lang, code) in fences(body) {
        if !matches!(lang.as_str(), "yaml" | "yml" | "") {
            continue;
        }
        if code.contains(&needle) {
            return synthesize_config(rule_id, &code);
        }
    }
    format!("rules:\n  - rule: {rule_id}\n    scope: repository\n")
}

fn synthesize_config(rule_id: &str, source: &str) -> String {
    let mut options = extract_options_block(source);
    if options.trim().is_empty() {
        return format!("rules:\n  - rule: {rule_id}\n    scope: repository\n");
    }
    if !options.ends_with('\n') {
        options.push('\n');
    }
    format!("rules:\n  - rule: {rule_id}\n    scope: repository\n    options:\n{options}")
}

fn extract_options_block(source: &str) -> String {
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "options:" {
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let child = indent + 2;
        let mut collected = Vec::new();
        for rest in lines.by_ref() {
            let rest_indent = rest.chars().take_while(|c| *c == ' ').count();
            if rest.trim().is_empty() {
                collected.push(rest.to_string());
                continue;
            }
            if rest_indent < child {
                break;
            }
            collected.push(format!(
                "{}{}",
                " ".repeat(6 + rest_indent - child),
                rest.trim_start()
            ));
        }
        return collected.join("\n");
    }
    String::new()
}

pub fn materialize_snippet(config: &str, snippet: &[(String, String)]) -> TempDir {
    // An explicitly named config is part of the example, including its project
    // catalog and effects; unnamed rule examples retain the synthesized config.
    let config = snippet
        .iter()
        .filter(|(lang, code)| {
            matches!(lang.as_str(), "yaml" | "yml") && starts_with_path_comment(lang, code)
        })
        .flat_map(|(lang, code)| split_named_files(code, lang, None))
        .find_map(|(name, body)| (name == ".no-mistakes.yml").then_some(body))
        .unwrap_or_else(|| config.to_owned());
    let dir = TempDir::new().expect("temp fixture");
    std::fs::write(dir.path().join(".no-mistakes.yml"), &config).expect("write config");
    if snippet.iter().any(|(lang, _)| {
        matches!(
            lang.as_str(),
            "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs"
        )
    }) {
        write_path(
            dir.path(),
            "tsconfig.json",
            "{\n  \"compilerOptions\": { \"strict\": true },\n  \"include\": [\"**/*\"]\n}\n",
        );
    }
    let catalog = schema_catalog_path(&config);
    for (lang, code) in snippet {
        if matches!(lang.as_str(), "yaml" | "yml")
            && code.contains("rules:")
            && !starts_with_path_comment(lang, code)
        {
            continue;
        }
        write_files(dir.path(), lang, code, catalog.as_deref());
    }
    if let Some(path) = catalog.as_deref() {
        let catalog_file = dir.path().join(path);
        if !catalog_file.exists() {
            write_path(
                dir.path(),
                path,
                "{\n  \"formatVersion\": 2,\n  \"coverage\": \"complete\",\n  \"tables\": {}\n}\n",
            );
        }
    }
    dir
}

fn schema_catalog_path(config: &str) -> Option<String> {
    config.lines().find_map(|line| {
        line.trim()
            .strip_prefix("schemaCatalogPath:")
            .map(|value| {
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string()
            })
            .filter(|value| !value.is_empty())
    })
}

fn write_files(root: &Path, lang: &str, code: &str, catalog: Option<&str>) {
    if lang == "text" && looks_like_path_listing(code) {
        for line in code.lines() {
            let path = line.trim();
            if path.is_empty() || path.starts_with('#') {
                continue;
            }
            write_path(root, path, "");
        }
        return;
    }
    let parts = split_named_files(code, lang, catalog);
    for (name, body) in parts {
        if name == ".no-mistakes.yml" {
            write_path(root, &name, &body);
        } else {
            write_path(root, &name, &wrap_workflow_yaml(lang, &body));
        }
    }
}

fn looks_like_path_listing(code: &str) -> bool {
    let lines: Vec<_> = code
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    !lines.is_empty()
        && lines.iter().all(|line| {
            !line.contains(' ')
                && !looks_like_email_line(line)
                && (line.contains('/') || line.contains('.'))
                && !line.contains('=')
        })
}

fn looks_like_email_line(line: &str) -> bool {
    // `owner@example.com` is fixture content. `app/@modal/page.tsx` and
    // `packages/@scope/pkg` still contain `/`, so they stay path listings.
    line.contains('@') && !line.contains('/')
}

pub fn starts_with_path_comment(lang: &str, code: &str) -> bool {
    code.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .is_some_and(|line| path_comment(line, path_comment_prefix(lang)).is_some())
}

/// Named-file fences in one Valid/Counterexample section are one fixture.
pub fn snippet_runs(fences: &[(String, String)]) -> Vec<(usize, Vec<(String, String)>)> {
    if fences.len() > 1
        && fences
            .iter()
            .all(|(lang, code)| starts_with_path_comment(lang, code))
    {
        return vec![(0, fences.to_vec())];
    }
    fences
        .iter()
        .enumerate()
        .map(|(index, fence)| (index, vec![fence.clone()]))
        .collect()
}

fn path_comment_prefix(lang: &str) -> &'static str {
    match lang {
        "sql" => "-- ",
        "yaml" | "yml" | "sh" | "bash" | "py" | "toml" => "# ",
        _ => "// ",
    }
}

fn split_named_files(code: &str, lang: &str, catalog: Option<&str>) -> Vec<(String, String)> {
    let comment = path_comment_prefix(lang);
    let mut files = Vec::new();
    let mut current_name = default_filename(lang, code, catalog);
    let mut current = String::new();
    for line in code.lines() {
        if let Some(name) = path_comment(line, comment) {
            if !current.trim().is_empty() {
                files.push((current_name, current.trim_end().to_string()));
            }
            current_name = name;
            current.clear();
            continue;
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        files.push((current_name, current.trim_end().to_string()));
    }
    if files.is_empty() {
        files.push((default_filename(lang, code, catalog), code.to_string()));
    }
    files
}

fn path_comment(line: &str, prefix: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(prefix)?.trim();
    if rest.is_empty() || rest.contains(' ') || !rest.contains('.') {
        return None;
    }
    if rest
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '_' | '-'))
    {
        Some(rest.to_string())
    } else {
        None
    }
}

fn default_filename(lang: &str, code: &str, catalog: Option<&str>) -> String {
    if matches!(lang, "json" | "jsonc") && code.contains("\"formatVersion\"") {
        if let Some(path) = catalog {
            return path.to_string();
        }
    }
    if matches!(lang, "json" | "jsonc") && code.contains("\"name\"") && code.contains("\"version\"")
    {
        return "package.json".into();
    }
    match lang {
        "rs" | "rust" => "src/lib.rs".into(),
        "swift" => "clients/App.swift".into(),
        "tsx" => "src/index.tsx".into(),
        "jsx" => "src/index.jsx".into(),
        "ts" | "mts" | "cts" => "src/index.ts".into(),
        "js" | "mjs" | "cjs" => "src/index.js".into(),
        "sql" => "db/schema.sql".into(),
        "json" | "jsonc" => "schema.json".into(),
        "yml" | "yaml"
            if code.contains("jobs:")
                || code.contains("uses: actions/")
                || code.contains("sparse-checkout") =>
        {
            ".github/workflows/ci.yml".into()
        }
        "yml" | "yaml" => "config.yml".into(),
        "md" | "markdown" if code.contains("# AGENTS.md") || code.contains("# CLAUDE.md") => {
            "AGENTS.md".into()
        }
        "md" | "markdown" => "README.md".into(),
        "cs" | "csharp" => "src/App.cs".into(),
        "sh" | "bash" => "scripts/run.sh".into(),
        "toml" => "Cargo.toml".into(),
        _ => "fixture.txt".into(),
    }
}

fn wrap_workflow_yaml(lang: &str, body: &str) -> String {
    if !matches!(lang, "yaml" | "yml") {
        return body.to_string();
    }
    if body.contains("jobs:") || body.contains("runs-on:") {
        return body.to_string();
    }
    if body.contains("runs:") && body.contains("using:") {
        return body.to_string();
    }
    if body.contains("uses:")
        || body.contains("timeout-minutes:")
        || body.contains("sparse-checkout")
    {
        return format!(
            "name: ci\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n{}",
            body.lines()
                .map(|line| format!("      {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    body.to_string()
}

fn write_path(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, body).expect("write snippet");
}

pub fn run_rule(root: &Path, rule_id: &str) -> Result<Vec<String>, String> {
    let config = root.join(".no-mistakes.yml");
    let mut hits = Vec::new();
    let mut errors = Vec::new();
    for result in [
        no_mistakes::codebase::rules::run_filesystem_rules(root, Some(&config)),
        no_mistakes::codebase::rules::run_check(root, Some(&config), None),
    ] {
        match result {
            Ok(findings) => hits.extend(
                findings
                    .into_iter()
                    .filter(|finding| finding.rule == rule_id),
            ),
            Err(err) => errors.push(err.to_string()),
        }
    }
    match no_mistakes::codebase::unique_exports::analyze_project(root, Some(&config), None) {
        Ok(findings) => hits.extend(
            findings
                .into_iter()
                .filter(|finding| finding.rule == rule_id)
                .map(|finding| no_mistakes::codebase::rules::RuleFinding {
                    rule: finding.rule,
                    file: finding.file,
                    line: finding.line as usize,
                    message: finding.message,
                    import: None,
                    target: None,
                }),
        ),
        Err(err) => errors.push(err.to_string()),
    }
    if hits.is_empty() && !errors.is_empty() {
        return Err(errors.join("; "));
    }
    hits.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });
    hits.dedup();
    Ok(hits
        .into_iter()
        .map(|finding| format!("{}:{}:{}", finding.file, finding.line, finding.message))
        .collect())
}

pub fn page_path(repo_root: &Path, rule_id: &str) -> PathBuf {
    repo_root.join("docs/rules").join(format!("{rule_id}.md"))
}

pub fn load_page(repo_root: &Path, rule_id: &str) -> RulePage {
    let file = format!("{rule_id}.md");
    let path = page_path(repo_root, rule_id);
    RulePage {
        file,
        body: std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display())),
    }
}
