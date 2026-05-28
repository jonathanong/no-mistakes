//! Queue-domain diff scanning: extracts `(binding, kind, name)` triples from
//! `.add(...)`, `.addBulk([...])`, `new Worker(...)`, `createQueue(...)`, and
//! `new Queue(...)` shapes on the removed side of a unified diff. Two-pass
//! over removed-only and removed∪context keeps multi-line shapes catchable
//! without false-flagging values that live purely on unchanged context.

use super::super::super::diff_parser::DiffFile;
use super::{join_with_context, scan_string_domain, QueueIdent, QUEUE_KIND_JOB, QUEUE_KIND_QUEUE};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

pub(super) const QUEUE_JOB_PATTERN: &str = concat!(
    // Capture the binding identifier of a `.add(...)` call (`emailQueue` in
    // `emailQueue.add("sync")`) so the diff side can scope job hints by the
    // calling binding. Other queue-defining shapes don't carry a relevant
    // binding — they *define* the queue.
    r#"(?:(?P<add_binding>\w+)?\s*\.add\s*\(\s*['"`](?P<add>[^'"`{}$\s]+)['"`]|"#,
    r#"\bnew\s+Worker\s*\(\s*['"`](?P<worker>[^'"`{}$\s]+)['"`]|"#,
    r#"\bcreateQueue\s*\(\s*['"`](?P<create>[^'"`{}$\s]+)['"`]|"#,
    r#"\bnew\s+Queue\s*\(\s*['"`](?P<queue>[^'"`{}$\s]+)['"`])"#,
);

// Separate regex applied only when a hunk contains `.addBulk(`. Captures
// every `name: "X"` inside the joined hunk text so multi-entry addBulk
// calls do not lose later entries. There can be false positives if an
// unrelated object literal in the same hunk has a `name:` key whose value
// matches a real removed job name; in practice that overlap is rare.
const QUEUE_ADDBULK_NAME_PATTERN: &str = r#"\bname\s*:\s*['"`](?P<name>[^'"`{}$\s]+)['"`]"#;

pub(super) fn scan_queue_add_calls(diff_files: &[DiffFile]) -> BTreeMap<PathBuf, Vec<QueueIdent>> {
    let mut out: BTreeMap<PathBuf, Vec<QueueIdent>> = BTreeMap::new();
    let Ok(re) = regex::Regex::new(QUEUE_JOB_PATTERN) else {
        return out;
    };
    let per_file: Vec<(PathBuf, Vec<QueueIdent>)> = diff_files
        .par_iter()
        .filter_map(|df| truly_removed_add_calls(&re, df))
        .collect();
    for (path, values) in per_file {
        out.entry(path).or_default().extend(values);
    }
    for values in out.values_mut() {
        values.sort();
        values.dedup();
    }
    out
}

fn truly_removed_add_calls(re: &regex::Regex, df: &DiffFile) -> Option<(PathBuf, Vec<QueueIdent>)> {
    if df.removed_lines.is_empty() {
        return None;
    }
    // Mirror the two-pass strategy in `truly_removed_strings`: a `-`-only
    // scan catches the common single-line case, and a `removed ∪ context`
    // scan catches multi-line shapes that need the call surroundings to
    // match. Values whose only occurrence is on a context line are NOT
    // truly removed and are subtracted via the `context_only` set.
    let removed_only = scan_add_calls(re, &df.removed_lines.join("\n"));
    let context_only: HashSet<(Option<String>, String)> =
        scan_add_calls(re, &df.context_lines.join("\n"))
            .into_iter()
            .collect();
    let removed_with_ctx =
        scan_add_calls(re, &join_with_context(&df.removed_lines, &df.context_lines));
    let added: HashSet<(Option<String>, String)> = scan_add_calls(re, &df.added_lines.join("\n"))
        .into_iter()
        .collect();
    let mut truly_removed: Vec<QueueIdent> = Vec::new();
    let mut seen: HashSet<(Option<String>, String)> = HashSet::new();
    for (binding, job) in removed_only {
        let key = (binding.clone(), job.clone());
        if added.contains(&key) {
            continue;
        }
        if seen.insert(key) {
            truly_removed.push((binding, QUEUE_KIND_JOB.to_string(), job));
        }
    }
    for (binding, job) in removed_with_ctx {
        let key = (binding.clone(), job.clone());
        if context_only.contains(&key) {
            continue;
        }
        if added.contains(&key) {
            continue;
        }
        if seen.insert(key) {
            truly_removed.push((binding, QUEUE_KIND_JOB.to_string(), job));
        }
    }
    (!truly_removed.is_empty()).then(|| (df.path.clone(), truly_removed))
}

fn scan_add_calls(re: &regex::Regex, joined: &str) -> Vec<(Option<String>, String)> {
    let mut out: Vec<(Option<String>, String)> = Vec::new();
    for caps in re.captures_iter(joined) {
        if let Some(job) = caps.name("add") {
            let binding = caps
                .name("add_binding")
                .map(|m| m.as_str().to_string())
                .filter(|s| !s.is_empty());
            out.push((binding, job.as_str().to_string()));
        }
    }
    out
}

pub(super) fn scan_addbulk_names_kinded(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<QueueIdent>> {
    // addBulk entries don't reliably carry a binding through the regex hop
    // (the `[{ name: ... }]` literal is separated from `binding.addBulk(...)`
    // by arbitrary content), so the binding is recorded as None. The
    // dependent side similarly keys addBulk-discovered jobs under `None` so
    // they continue to line up.
    scan_addbulk_names(diff_files)
        .into_iter()
        .map(|(path, names)| {
            (
                path,
                names
                    .into_iter()
                    .map(|n| (None, QUEUE_KIND_JOB.to_string(), n))
                    .collect(),
            )
        })
        .collect()
}

pub(super) fn scan_queue_defining_shapes(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<QueueIdent>> {
    scan_string_domain(
        diff_files,
        QUEUE_JOB_PATTERN,
        &["worker", "create", "queue"],
    )
    .into_iter()
    .map(|(path, names)| {
        (
            path,
            names
                .into_iter()
                .map(|n| (None, QUEUE_KIND_QUEUE.to_string(), n))
                .collect(),
        )
    })
    .collect()
}

fn scan_addbulk_names(diff_files: &[DiffFile]) -> BTreeMap<PathBuf, Vec<String>> {
    let mut out: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    let Ok(name_re) = regex::Regex::new(QUEUE_ADDBULK_NAME_PATTERN) else {
        return out;
    };
    let per_file: Vec<(PathBuf, Vec<String>)> = diff_files
        .par_iter()
        .filter_map(|df| addbulk_names_for_file(&name_re, df))
        .collect();
    for (path, names) in per_file {
        out.entry(path).or_default().extend(names);
    }
    for values in out.values_mut() {
        values.sort();
        values.dedup();
    }
    out
}

fn addbulk_names_for_file(name_re: &regex::Regex, df: &DiffFile) -> Option<(PathBuf, Vec<String>)> {
    if df.removed_lines.is_empty() {
        return None;
    }
    // The addBulk shape is inherently multi-line, so the primary scan runs
    // over the joined removed ∪ context buffer. To avoid the trap where a
    // `name: "x"` literal that only exists on an unchanged context line
    // gets reported as removed, names matched in context-only get
    // subtracted alongside the usual `+`-side subtraction.
    let removed_joined = join_with_context(&df.removed_lines, &df.context_lines);
    if !removed_joined.contains(".addBulk") {
        return None;
    }
    let context_joined = df.context_lines.join("\n");
    let added_joined = df.added_lines.join("\n");
    let context_only: HashSet<String> = name_re
        .captures_iter(&context_joined)
        .filter_map(|c| c.name("name").map(|m| m.as_str().to_string()))
        .collect();
    let mut added_names: HashSet<String> = HashSet::new();
    if added_joined.contains(".addBulk") {
        for caps in name_re.captures_iter(&added_joined) {
            if let Some(m) = caps.name("name") {
                added_names.insert(m.as_str().to_string());
            }
        }
    }
    // Subtract job names that surface as `.add('x', ...)` on the `+` side:
    // refactoring `addBulk([{ name: 'x' }])` into `add('x', payload)` should
    // not register `x` as a removed job.
    if let Ok(add_re) = regex::Regex::new(r#"\.add\s*\(\s*['"`](?P<add>[^'"`{}$\s]+)['"`]"#) {
        for caps in add_re.captures_iter(&added_joined) {
            if let Some(m) = caps.name("add") {
                added_names.insert(m.as_str().to_string());
            }
        }
    }
    let mut names: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for caps in name_re.captures_iter(&removed_joined) {
        if let Some(m) = caps.name("name") {
            let value = m.as_str().to_string();
            if context_only.contains(&value) {
                continue;
            }
            if added_names.contains(&value) {
                continue;
            }
            if seen.insert(value.clone()) {
                names.push(value);
            }
        }
    }
    (!names.is_empty()).then(|| (df.path.clone(), names))
}
