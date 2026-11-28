//! Active record state and per-line dispatch for `DA:` / `BRDA:` / function records.

use super::functions::{parse_fn, parse_fna, parse_fnda, parse_fnl};
use crate::model::{BranchKey, FileCoverage};
use std::collections::BTreeMap;

pub(super) struct ActiveRecord {
    pub(super) path: Option<String>,
    pub(super) coverage: FileCoverage,
    pub(super) fnl_index: BTreeMap<u32, (u32, Option<u32>)>,
    pub(super) empty_placeholder: bool,
}

impl ActiveRecord {
    pub(super) fn new(path: String) -> Self {
        Self {
            path: Some(path),
            coverage: FileCoverage::default(),
            fnl_index: BTreeMap::new(),
            empty_placeholder: false,
        }
    }

    pub(super) fn skipped() -> Self {
        Self {
            path: None,
            coverage: FileCoverage::default(),
            fnl_index: BTreeMap::new(),
            empty_placeholder: false,
        }
    }
}

pub(super) enum DataRecord<'a> {
    Da(&'a str),
    Fn(&'a str),
    Fnda(&'a str),
    Fnl(&'a str),
    Fna(&'a str),
    Brda(&'a str),
}

pub(super) fn classify_data_record(line: &str) -> Option<DataRecord<'_>> {
    if let Some(rest) = line.strip_prefix("DA:") {
        Some(DataRecord::Da(rest))
    } else if let Some(rest) = line.strip_prefix("FN:") {
        Some(DataRecord::Fn(rest))
    } else if let Some(rest) = line.strip_prefix("FNDA:") {
        Some(DataRecord::Fnda(rest))
    } else if let Some(rest) = line.strip_prefix("FNL:") {
        Some(DataRecord::Fnl(rest))
    } else if let Some(rest) = line.strip_prefix("FNA:") {
        Some(DataRecord::Fna(rest))
    } else {
        line.strip_prefix("BRDA:").map(DataRecord::Brda)
    }
}

pub(super) fn apply_data_record(
    record: DataRecord<'_>,
    active: &mut ActiveRecord,
) -> Result<(), &'static str> {
    match record {
        DataRecord::Da(rest) => parse_da(rest, &mut active.coverage),
        DataRecord::Fn(rest) => parse_fn(rest, active),
        DataRecord::Fnda(rest) => parse_fnda(rest, active),
        DataRecord::Fnl(rest) => parse_fnl(rest, active),
        DataRecord::Fna(rest) => parse_fna(rest, active),
        DataRecord::Brda(rest) => parse_brda(rest, &mut active.coverage),
    }
}

fn parse_da(rest: &str, cov: &mut FileCoverage) -> Result<(), &'static str> {
    let (line_raw, after_line) = rest.split_once(',').ok_or("malformed DA record")?;
    let hits_raw = after_line
        .split_once(',')
        .map_or(after_line, |(hits, _)| hits);
    let line_no = line_raw
        .parse::<u32>()
        .map_err(|_| "invalid DA line number")?;
    let hits = hits_raw
        .parse::<u64>()
        .map_err(|_| "invalid DA hit count")?;
    let entry = cov.lines.entry(line_no).or_insert(0);
    *entry = entry.saturating_add(hits);
    Ok(())
}

fn parse_brda(rest: &str, cov: &mut FileCoverage) -> Result<(), &'static str> {
    let (line_raw, r1) = rest.split_once(',').ok_or("malformed BRDA record")?;
    let (block, r2) = r1.split_once(',').ok_or("malformed BRDA record")?;
    let (branch, r3) = r2.split_once(',').ok_or("malformed BRDA record")?;
    let taken_raw = r3.split_once(',').map_or(r3, |(taken, _)| taken);

    let line_no = line_raw
        .parse::<u32>()
        .map_err(|_| "invalid BRDA line number")?;
    if block.is_empty() || branch.is_empty() {
        return Err("empty BRDA block or branch id");
    }
    let taken = if taken_raw == "-" {
        0
    } else {
        taken_raw
            .parse::<u64>()
            .map_err(|_| "invalid BRDA taken count")?
    };

    let key = BranchKey::new(line_no, block, branch);
    let entry = cov.branches.entry(key).or_insert(0);
    *entry = entry.saturating_add(taken);
    Ok(())
}
