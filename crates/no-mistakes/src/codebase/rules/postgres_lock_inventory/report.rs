use super::scan::FileInventory;
use super::{CompiledOptions, LockInventoryReport, LockInventorySite, UnanalyzedLockSite};
use std::collections::BTreeMap;

pub(super) fn build(
    options: &CompiledOptions,
    mut rows: Vec<(String, FileInventory)>,
) -> LockInventoryReport {
    rows.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    let mut report = LockInventoryReport {
        sites: Vec::new(),
        counts: BTreeMap::new(),
        unanalyzed: Vec::new(),
        baseline_file: options.baseline_file.clone(),
        added: Vec::new(),
        removed: Vec::new(),
        baseline_updated: false,
    };
    for (file, row) in rows {
        append_file(&mut report, &file, row);
    }
    report.unanalyzed.sort_unstable();
    report
}

fn append_file(report: &mut LockInventoryReport, file: &str, mut row: FileInventory) {
    row.sites.sort_unstable();
    let mut ordinals = BTreeMap::new();
    for site in &row.sites {
        let (hash, ordinal) = ordinals
            .entry((&site.category, &site.fingerprint))
            .or_insert_with(|| (fingerprint(&[file, &site.category, &site.fingerprint]), 0));
        *ordinal += 1;
        *report.counts.entry(site.category.clone()).or_insert(0) += 1;
        report.sites.push(LockInventorySite {
            file: file.into(),
            line: site.line.max(1),
            category: site.category.clone(),
            key: format!("{hash:032x}#{ordinal}"),
        });
    }
    report.unanalyzed.extend(
        row.gaps
            .into_iter()
            .map(|(line, reason)| UnanalyzedLockSite {
                file: file.into(),
                line: line.max(1),
                reason: reason.into(),
            }),
    );
}

fn fingerprint(parts: &[&str]) -> u128 {
    const FNV_OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
    const FNV_PRIME: u128 = 0x0000000001000000000000000000013b;
    let bytes = parts.iter().flat_map(|part| {
        (part.len() as u64)
            .to_le_bytes()
            .into_iter()
            .chain(part.bytes())
    });
    bytes.fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u128::from(byte)).wrapping_mul(FNV_PRIME)
    })
}
