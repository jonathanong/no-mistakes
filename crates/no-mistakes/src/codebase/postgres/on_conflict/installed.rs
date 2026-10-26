use std::path::Path;

use crate::codebase::postgres::statement_facts::SqlTriggerFact;

/// Triggers installed at `insert_line` in `insert_path`.
///
/// Facts with an empty `source_path` are the unordered slices unit tests pass
/// to `judge_file`. Those keep every create and ignore drops. Positioned facts
/// honor path order, then line order, with a same-line create before a drop.
pub(super) fn at(
    triggers: &[SqlTriggerFact],
    insert_path: &Path,
    insert_line: usize,
) -> Vec<SqlTriggerFact> {
    if triggers
        .iter()
        .all(|trigger| trigger.source_path.as_os_str().is_empty())
    {
        return triggers
            .iter()
            .filter(|trigger| !trigger.removed)
            .cloned()
            .collect();
    }
    let mut events: Vec<&SqlTriggerFact> = triggers.iter().collect();
    events.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then(left.line.cmp(&right.line))
            .then(left.removed.cmp(&right.removed))
    });
    let mut always = Vec::new();
    let mut active = Vec::new();
    for trigger in events {
        if trigger.source_path.as_os_str().is_empty() {
            if !trigger.removed {
                always.push(trigger.clone());
            }
            continue;
        }
        if !earlier(trigger, insert_path, insert_line) {
            continue;
        }
        if trigger.removed {
            active.retain(|installed| !same_trigger(installed, trigger));
        } else if let Some(installed) = active
            .iter_mut()
            .find(|installed| same_trigger(installed, trigger))
        {
            *installed = trigger.clone();
        } else {
            active.push(trigger.clone());
        }
    }
    always.extend(active);
    always
}

fn earlier(trigger: &SqlTriggerFact, path: &Path, line: usize) -> bool {
    match trigger.source_path.as_path().cmp(path) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Equal => trigger.line < line,
        std::cmp::Ordering::Greater => false,
    }
}

fn same_trigger(installed: &SqlTriggerFact, event: &SqlTriggerFact) -> bool {
    installed.name.eq_ignore_ascii_case(&event.name)
        && (event.table.is_empty() || installed.table.eq_ignore_ascii_case(&event.table))
}
