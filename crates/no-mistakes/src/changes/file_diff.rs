use super::attributes;
use super::report::{path_change, Observation};
use super::rules::{Id, Rule, Rules};
use super::Snapshot;
use crate::codebase::keyed_diff;
use std::cell::OnceCell;
use std::sync::Arc;

#[derive(Default)]
pub(super) struct FileDiff<'a> {
    pub categories: Vec<(&'a str, Observation)>,
    pub unclassified: Option<String>,
    pub changed_file: Option<String>,
    pub reasons: Vec<String>,
    pub deferred: Vec<(&'a str, String)>,
}

pub(super) fn compare<'a>(
    path: &str,
    base: &Snapshot,
    head: &Snapshot,
    rules: &Rules<'a>,
) -> Option<FileDiff<'a>> {
    if !base.knows(path) || !head.knows(path) {
        return None;
    }
    let selector_matches = rules.selectors.matches(path);
    let matches: Vec<_> = selector_matches
        .iter()
        .map(|&index| &rules.rules[index])
        .filter(|rule| !rule.is_excluded(path, &rules.excludes))
        .collect();
    let excluded_by_category = matches.is_empty() && !selector_matches.is_empty();
    match compare_readable(path, base, head, rules, &matches, excluded_by_category) {
        Ok(diff) => diff,
        Err(reason) => Some(FileDiff {
            reasons: vec![reason],
            ..FileDiff::default()
        }),
    }
}

struct FileBytes<'a> {
    old: Option<&'a [u8]>,
    new: Option<&'a [u8]>,
    unchanged: bool,
    link: bool,
}

fn compare_readable<'a>(
    path: &str,
    base: &Snapshot,
    head: &Snapshot,
    rules: &Rules<'a>,
    matches: &[&Rule<'a>],
    excluded_by_category: bool,
) -> Result<Option<FileDiff<'a>>, String> {
    if base.is_link(path) || head.is_link(path) {
        let old = base.link_identity(path)?;
        let new = head.link_identity(path)?;
        let bytes = FileBytes {
            unchanged: old == new,
            old: old.as_deref(),
            new: new.as_deref(),
            link: true,
        };
        return Ok(classify(path, rules, matches, excluded_by_category, bytes));
    }
    let base_source = base.read(path)?;
    let head_source = head.read(path)?;
    let old = base.bytes(path, &base_source)?;
    let new = head.bytes(path, &head_source)?;
    let bytes = FileBytes {
        old,
        new,
        unchanged: old == new,
        link: false,
    };
    Ok(classify(path, rules, matches, excluded_by_category, bytes))
}

fn classify<'a>(
    path: &str,
    rules: &Rules<'a>,
    matches: &[&Rule<'a>],
    excluded_by_category: bool,
    bytes: FileBytes<'_>,
) -> Option<FileDiff<'a>> {
    let FileBytes {
        old,
        new,
        unchanged,
        link,
    } = bytes;
    if matches.is_empty() {
        if unchanged {
            return None;
        }
        return Some(if excluded_by_category {
            FileDiff {
                unclassified: Some(path.to_owned()),
                ..FileDiff::default()
            }
        } else {
            FileDiff {
                changed_file: Some(path.to_owned()),
                ..FileDiff::default()
            }
        });
    }
    let mut result = FileDiff::default();
    let mut parsed = vec![None; rules.keyed.len()];
    let attribute_paths = OnceCell::new();
    let attributes = |enabled: bool| -> Option<Arc<[String]>> {
        enabled
            .then(|| {
                attribute_paths.get_or_init(|| {
                    (!unchanged && !link && path.ends_with(".json"))
                        .then(|| attributes::diff(old?, new?).map(Arc::from))
                        .flatten()
                })
            })?
            .clone()
    };
    let path_obs = |id, emit| path_change(id, old, new, unchanged, attributes(emit));
    for &rule in matches {
        let changes = match rule.id {
            Id::Path => path_obs(path.to_owned(), rule.emit_attribute_paths),
            Id::FromPath {
                prefix,
                suffix,
                separator,
            } => {
                let Some(id) = path
                    .strip_prefix(prefix)
                    .and_then(|rest| rest.strip_suffix(suffix))
                    .filter(|id| !id.is_empty())
                else {
                    let reason = format!(
                        "path {path:?} does not match fromPath for changes category {:?}",
                        rule.category
                    );
                    result.record_error(rule.category, reason, unchanged);
                    continue;
                };
                let id =
                    separator.map_or_else(|| id.to_owned(), |separator| id.replace('/', separator));
                path_obs(id, rule.emit_attribute_paths)
            }
            Id::Keys(_) if link => {
                if !unchanged {
                    result.unclassified = Some(path.to_owned());
                }
                continue;
            }
            Id::Keys(index) => {
                let changes = parsed[index].get_or_insert_with(|| {
                    let (changes, [before, after]) =
                        keyed_diff::diff_with_presence(&rules.keyed[index], old, new)
                            .map_err(|error| format!("{error:#}"))?;
                    Ok::<_, String>(Observation {
                        touched: changes.all_ids().collect(),
                        before,
                        after,
                        attribute_paths: None,
                    })
                });
                match changes {
                    Ok(changes) => changes.clone(),
                    Err(error) => {
                        let reason = format!(
                            "cannot diff {path:?} for changes category {:?}: {error}",
                            rule.category
                        );
                        result.record_error(rule.category, reason, unchanged);
                        continue;
                    }
                }
            }
        };
        result.categories.push((rule.category, changes));
    }
    Some(result)
}

impl<'a> FileDiff<'a> {
    fn record_error(&mut self, category: &'a str, reason: String, unchanged: bool) {
        if unchanged {
            self.deferred.push((category, reason));
        } else {
            self.reasons.push(reason);
        }
    }
}
