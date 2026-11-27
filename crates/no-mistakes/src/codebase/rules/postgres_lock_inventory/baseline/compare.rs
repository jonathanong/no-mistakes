use super::super::{LockInventoryBaseline, LockInventoryReport, LockInventoryUnanalyzedDelta};
use super::identity;
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::codebase::rules::postgres_lock_inventory) fn compare(
    report: &mut LockInventoryReport,
    baseline: Option<&LockInventoryBaseline>,
) {
    compare_unanalyzed(report, baseline);
    let previous: BTreeMap<_, _> = baseline
        .into_iter()
        .flat_map(|baseline| &baseline.sites)
        .map(|site| (identity(site), site))
        .collect();
    let mut current = BTreeSet::new();
    for site in &report.sites {
        let entry = (
            site.file.as_str(),
            site.category.as_str(),
            site.key.as_str(),
        );
        if !previous.contains_key(&entry) {
            report.added.push(site.clone());
        }
        current.insert(entry);
    }
    let allow_removals = if report.baseline_unanalyzed {
        report.unanalyzed_added.is_empty()
    } else {
        report.unanalyzed.is_empty()
    };
    if allow_removals {
        report.removed.extend(
            previous
                .into_iter()
                .filter(|(key, _)| !current.contains(key))
                .map(|(_, entry)| entry.clone()),
        );
    }
}

fn compare_unanalyzed(report: &mut LockInventoryReport, baseline: Option<&LockInventoryBaseline>) {
    let previous: BTreeMap<(&str, &str), usize> = baseline
        .into_iter()
        .flat_map(|baseline| &baseline.unanalyzed)
        .map(|entry| ((entry.file.as_str(), entry.reason.as_str()), entry.count))
        .collect();
    if !report.baseline_unanalyzed {
        if report.unanalyzed.is_empty() {
            for ((file, reason), baseline_count) in previous {
                report.unanalyzed_removed.push(LockInventoryUnanalyzedDelta {
                    file: file.into(),
                    reason: reason.into(),
                    baseline_count,
                    current_count: 0,
                });
            }
        }
        return;
    }
    let mut current: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for gap in &report.unanalyzed {
        *current
            .entry((gap.file.as_str(), gap.reason.as_str()))
            .or_insert(0) += 1;
    }
    let all_keys: BTreeSet<(&str, &str)> = previous
        .keys()
        .copied()
        .chain(current.keys().copied())
        .collect();
    for key @ (file, reason) in all_keys {
        let baseline_count = previous.get(&key).copied().unwrap_or(0);
        let current_count = current.get(&key).copied().unwrap_or(0);
        if current_count > baseline_count {
            report.unanalyzed_added.push(LockInventoryUnanalyzedDelta {
                file: file.into(),
                reason: reason.into(),
                baseline_count,
                current_count,
            });
        } else if current_count < baseline_count {
            report.unanalyzed_removed.push(LockInventoryUnanalyzedDelta {
                file: file.into(),
                reason: reason.into(),
                baseline_count,
                current_count,
            });
        }
    }
}
