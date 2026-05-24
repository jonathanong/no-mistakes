use super::{is_code_file, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use globset::GlobSet;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct ScanCtx<'a> {
    pub(super) req_file: String,
    pub(super) ext: Vec<&'a str>,
    pub(super) excl: Vec<&'a str>,
    pub(super) globs: GlobSet,
    pub(super) file_set: HashSet<&'a PathBuf>,
    pub(super) files: &'a [PathBuf],
}

pub(super) fn scan_pkg(root: &Path, pkg_root: &Path, ctx: &ScanCtx) -> Vec<RuleFinding> {
    let subdirs: BTreeSet<String> = ctx
        .files
        .iter()
        .filter_map(|file| {
            let rel = file.strip_prefix(pkg_root).ok()?;
            let comps: Vec<&str> = rel
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect();
            if comps.len() >= 2 && is_code_file(file, &ctx.ext, &ctx.excl, &ctx.globs) {
                Some(comps[0].to_string())
            } else {
                None
            }
        })
        .collect();
    subdirs
        .into_iter()
        .filter_map(|subdir| {
            if ctx
                .file_set
                .contains(&pkg_root.join(&subdir).join(&ctx.req_file))
            {
                return None;
            }
            let dir_rel = relative_slash_path(root, &pkg_root.join(&subdir));
            Some(RuleFinding {
                rule: RULE_ID.to_string(),
                file: dir_rel.clone(),
                line: 1,
                message: format!(
                    "{dir_rel}: code-owning directory is missing {}",
                    ctx.req_file
                ),
                import: None,
                target: None,
            })
        })
        .collect()
}
