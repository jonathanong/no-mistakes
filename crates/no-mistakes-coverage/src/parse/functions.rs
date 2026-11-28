//! Parsing for `FN:`, `FNDA:`, `FNL:`, and `FNA:` LCOV function records.

use super::records::ActiveRecord;
use crate::model::FunctionCoverage;

const EMPTY_REPORT_MARKER: &str = "(empty-report)";

fn record_fn_def(active: &mut ActiveRecord, start: u32, end: Option<u32>, name: &str) {
    if name == EMPTY_REPORT_MARKER {
        active.empty_placeholder = true;
        return;
    }
    let entry = active
        .coverage
        .functions
        .entry(name.to_string())
        .or_insert_with(|| FunctionCoverage {
            line: Some(start),
            end_line: end,
            hits: 0,
        });
    if entry.line.is_none() {
        entry.line = Some(start);
    }
    if entry.end_line.is_none() {
        entry.end_line = end;
    }
}

fn record_fn_hits(active: &mut ActiveRecord, hits: u64, name: &str) {
    if name == EMPTY_REPORT_MARKER {
        active.empty_placeholder = true;
        return;
    }
    let entry = active
        .coverage
        .functions
        .entry(name.to_string())
        .or_default();
    entry.hits = entry.hits.saturating_add(hits);
}

pub(super) fn parse_fn(rest: &str, active: &mut ActiveRecord) -> Result<(), &'static str> {
    let (start_raw, after_start) = rest.split_once(',').ok_or("malformed FN record")?;
    let start = start_raw
        .parse::<u32>()
        .map_err(|_| "invalid FN start line")?;
    let (end, name) = match after_start.split_once(',') {
        Some((maybe_end, after_end)) => match maybe_end.parse::<u32>() {
            Ok(end_line) => (Some(end_line), after_end),
            Err(_) => (None, after_start),
        },
        None => (None, after_start),
    };
    if name.is_empty() {
        return Err("empty FN function name");
    }
    record_fn_def(active, start, end, name);
    Ok(())
}

pub(super) fn parse_fnda(rest: &str, active: &mut ActiveRecord) -> Result<(), &'static str> {
    let (hits_raw, name) = rest.split_once(',').ok_or("malformed FNDA record")?;
    let hits = hits_raw
        .parse::<u64>()
        .map_err(|_| "invalid FNDA hit count")?;
    if name.is_empty() {
        return Err("empty FNDA function name");
    }
    record_fn_hits(active, hits, name);
    Ok(())
}

pub(super) fn parse_fnl(rest: &str, active: &mut ActiveRecord) -> Result<(), &'static str> {
    let (first_raw, after_first) = rest.split_once(',').ok_or("malformed FNL record")?;
    let first = first_raw.parse::<u32>().map_err(|_| "invalid FNL number")?;
    match after_first.split_once(',') {
        None => {
            let start = after_first
                .parse::<u32>()
                .map_err(|_| "invalid FNL start line")?;
            active.fnl_index.insert(first, (start, None));
            Ok(())
        }
        Some((second_raw, third)) => {
            let second = second_raw
                .parse::<u32>()
                .map_err(|_| "invalid FNL second field")?;
            if let Ok(end) = third.parse::<u32>() {
                active.fnl_index.insert(first, (second, Some(end)));
                Ok(())
            } else if !third.is_empty() {
                record_fn_def(active, first, Some(second), third);
                Ok(())
            } else {
                Err("empty FNL function name")
            }
        }
    }
}

pub(super) fn parse_fna(rest: &str, active: &mut ActiveRecord) -> Result<(), &'static str> {
    if let Some((index_raw, after_index)) = rest.split_once(',') {
        if let Some((hits_raw, name)) = after_index.split_once(',') {
            if let (Ok(index), Ok(hits)) = (index_raw.parse::<u32>(), hits_raw.parse::<u64>()) {
                if !name.is_empty() {
                    if let Some(&(start, end)) = active.fnl_index.get(&index) {
                        record_fn_def(active, start, end, name);
                    }
                    record_fn_hits(active, hits, name);
                    return Ok(());
                }
            }
        }
    }
    parse_fnda(rest, active)
}
