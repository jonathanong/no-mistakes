use super::Options;
use std::path::{Path, PathBuf};

const SHEBANG_BYTES: usize = 256;

pub(crate) fn filtered_shell_files(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    rule_filter: &super::super::path_filter::RulePathFilter,
) -> Vec<PathBuf> {
    let mut candidates = collect_shell_files(root, opts, files, target_roots);
    candidates.retain(|path| rule_filter.is_match(path));
    candidates
}

pub(crate) fn collect_shell_files(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let sh = |p: &Path| p.extension().and_then(|e| e.to_str()) == Some("sh");
    let mut candidates: Vec<PathBuf> = files.iter().filter(|p| sh(p)).cloned().collect();
    for dir_rel in &opts.shebang_dirs {
        let dir = if dir_rel.is_empty() {
            root.to_path_buf()
        } else {
            root.join(dir_rel)
        };
        for path in files {
            if !path.starts_with(&dir) || sh(path) {
                continue;
            }
            if has_bash_shebang(path) {
                candidates.push(path.clone());
            }
        }
    }
    for rel in &opts.shell_files {
        let abs = root.join(rel);
        let in_scope = target_roots.is_empty() || target_roots.iter().any(|r| abs.starts_with(r));
        if abs.exists() && in_scope {
            candidates.push(abs);
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

pub(crate) fn has_bash_shebang(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; SHEBANG_BYTES];
    let n = file.read(&mut buf).unwrap_or(0);
    let header = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let l = header.lines().next().unwrap_or("");
    l.starts_with("#!/bin/bash")
        || l.starts_with("#!/usr/bin/env bash")
        || l.starts_with("#!/bin/sh")
        || l.starts_with("#!/usr/bin/env sh")
}
