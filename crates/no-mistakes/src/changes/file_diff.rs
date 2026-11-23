use super::report::Observation;
use super::rules::{Id, Rules};
use super::Snapshot;
use crate::codebase::keyed_diff;

#[derive(Default)]
pub(super) struct FileDiff<'a> {
    pub categories: Vec<(&'a str, Observation)>,
    pub unclassified: Option<String>,
    pub reasons: Vec<String>,
    pub deferred: Vec<(&'a str, String)>,
}

pub(super) fn compare<'a>(
    path: &str,
    base: &Snapshot,
    head: &Snapshot,
    rules: &Rules<'a>,
) -> Option<FileDiff<'a>> {
    if rules.excludes.is_match(path) || !base.knows(path) || !head.knows(path) {
        return None;
    }
    match compare_readable(path, base, head, rules) {
        Ok(diff) => diff,
        Err(reason) => Some(FileDiff {
            reasons: vec![reason],
            ..FileDiff::default()
        }),
    }
}

fn compare_readable<'a>(
    path: &str,
    base: &Snapshot,
    head: &Snapshot,
    rules: &Rules<'a>,
) -> Result<Option<FileDiff<'a>>, String> {
    let base_source = base.read(path)?;
    let head_source = head.read(path)?;
    let old = base.bytes(path, &base_source)?;
    let new = head.bytes(path, &head_source)?;
    let unchanged = old == new;
    let mut result = FileDiff::default();
    let matches = rules.selectors.matches(path);
    if matches.is_empty() {
        if unchanged {
            return Ok(None);
        }
        result.unclassified = Some(path.to_owned());
        return Ok(Some(result));
    }
    let mut parsed = vec![None; rules.keyed.len()];
    for index in matches {
        let rule = &rules.rules[index];
        let changes = match rule.id {
            Id::Path => path_change(path.to_owned(), old, new, unchanged),
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
                path_change(id, old, new, unchanged)
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
    Ok(Some(result))
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

fn path_change(id: String, old: Option<&[u8]>, new: Option<&[u8]>, unchanged: bool) -> Observation {
    Observation {
        before: old.map(|_| id.clone()).into_iter().collect(),
        after: new.map(|_| id.clone()).into_iter().collect(),
        touched: (!unchanged).then_some(id).into_iter().collect(),
    }
}
