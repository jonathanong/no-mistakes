use super::fragments::Fragment;
use super::modules::import_hit_fragments;
use super::resolve::FileCtx;
use super::tags::imported;
use crate::codebase::dependencies::extract::ImportedBindingKind;

/// A value use of an `import *` binding reads the module object, so every
/// exported fragment of that module escapes. A named or default import is
/// left to the call and binding scans: a sole `.append` argument must stay
/// silent.
pub(super) fn value_sites(ctx: &FileCtx<'_>) -> Vec<(u32, Vec<Fragment>)> {
    let mut sites = Vec::new();
    for (name, offset) in &ctx.ts.namespaces.value_use_offsets {
        let Some(import) = imported(ctx.ts, name) else {
            continue;
        };
        if import.kind != ImportedBindingKind::Namespace {
            continue;
        }
        let found =
            import_hit_fragments(ctx.catalog, ctx.graph, ctx.facts, ctx.file, import, false);
        if found.is_empty() {
            continue;
        }
        sites.push((*offset, found));
    }
    sites
}
