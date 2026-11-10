//! Line of an unresolved `setupFiles` entry. Only consulted when setups are unknown.
use crate::codebase::dependencies::graph::TsFactLookup;
use std::path::{Path, PathBuf};

pub(super) fn setup_hole(facts: &dyn TsFactLookup, files: &[PathBuf]) -> Option<(PathBuf, u32)> {
    let mut best = None;
    for file in files {
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !name.contains("vitest.config") && !name.contains("vite.config") {
            continue;
        }
        let Some(source) = facts
            .get_ts_facts(file)
            .and_then(|facts| facts.source.as_deref())
        else {
            continue;
        };
        let Some(index) = source.find("setupFiles") else {
            continue;
        };
        remember(
            &mut best,
            file,
            crate::codebase::ts_source::byte_offset_to_line(source, index),
        );
    }
    best
}

fn remember(slot: &mut Option<(PathBuf, u32)>, file: &Path, start: u32) {
    let next = (file.to_path_buf(), start);
    if slot.as_ref().is_none_or(|prev| next < *prev) {
        *slot = Some(next);
    }
}
