//! Reverse JSX usage query: given a component file (optionally `path#Symbol`),
//! find every JSX callsite that renders it, the stories/tests that import it, and
//! the prop type names it exports.

mod helpers;

use crate::ast;
use crate::imports::relative_string;
use crate::react_traits::analyze::import_table::build_import_table;
use crate::react_traits::analyze::jsx_callsites::collect_jsx_callsites;
use crate::react_traits::pipeline::run::discover_react_files;
use crate::react_traits::report::types::{Callsite, RootConfig, UsagesReport, UsagesTarget};
use anyhow::Result;
use helpers::{
    callsite_symbol_matches, filter_importers, importer_symbol_matches, is_story, is_test,
    prop_type_names, same_path, split_target,
};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub use helpers::UsagesInclude;

struct FileHit {
    callsites: Vec<Callsite>,
    importer: Option<String>,
}

pub fn run_usages(
    root: &Path,
    config_path: Option<&Path>,
    target: &str,
    scan_targets: &[String],
    include: &UsagesInclude,
) -> Result<UsagesReport> {
    let root_config: RootConfig = crate::config::load_config(root, config_path, &[".no-mistakes"])?;
    let file_config = root_config.into_file_config();

    let (path_part, symbol) = split_target(target);
    let candidate = if Path::new(path_part).is_absolute() {
        PathBuf::from(path_part)
    } else {
        root.join(path_part)
    };
    if !candidate.exists() {
        anyhow::bail!("target file not found: {}", candidate.display());
    }
    // `exists()` above guarantees this resolves; `?` keeps the rare race as an error.
    let target_abs = candidate.canonicalize()?;

    let files = discover_react_files(root, &file_config, scan_targets)?;
    let hits: Vec<FileHit> = files
        .par_iter()
        .filter_map(|file| analyze_one(file, root, &target_abs, symbol.as_deref()).ok())
        .collect();

    let mut callsites = Vec::new();
    let mut importer_files = BTreeSet::new();
    for hit in hits {
        callsites.extend(hit.callsites);
        if let Some(file) = hit.importer {
            importer_files.insert(file);
        }
    }
    callsites.sort_by(|a, b| (a.file.as_str(), a.line).cmp(&(b.file.as_str(), b.line)));

    let stories = include
        .stories
        .then(|| filter_importers(&importer_files, is_story));
    let tests = include
        .tests
        .then(|| filter_importers(&importer_files, is_test));
    let prop_types = include.prop_types.then(|| prop_type_names(&candidate));

    Ok(UsagesReport {
        target: UsagesTarget {
            file: relative_string(root, &candidate),
            symbol,
        },
        callsites,
        stories,
        tests,
        prop_types,
    })
}

fn analyze_one(
    file: &Path,
    root: &Path,
    target_abs: &Path,
    symbol: Option<&str>,
) -> Result<FileHit> {
    let source = std::fs::read_to_string(file)?;
    ast::with_program(file, &source, |program, _src| {
        let import_table = build_import_table(file, program);
        let importer = import_table
            .values()
            .any(|entry| {
                same_path(&entry.resolved_path, target_abs)
                    && importer_symbol_matches(&entry.exported_name, symbol)
            })
            .then(|| relative_string(root, file));

        let callsites = collect_jsx_callsites(program, &import_table, &file.to_path_buf(), &source)
            .into_iter()
            .filter(|c| {
                same_path(&c.resolved_path, target_abs)
                    && callsite_symbol_matches(&c.exported_name, symbol)
            })
            .map(|c| Callsite {
                file: relative_string(root, file),
                line: c.line,
                component: c.exported_name,
                props: c.props,
                has_spread: c.has_spread,
            })
            .collect();

        FileHit {
            callsites,
            importer,
        }
    })
}

#[cfg(test)]
mod tests;
