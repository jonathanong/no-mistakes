/// Build an ignore walker that keeps gitignore semantics without leaking
/// ignore files from outside the nearest git root.
///
/// `require_git(false)` is required for ad-hoc directories that have a
/// `.gitignore` but no `.git` metadata. Combined with default `parents(true)`,
/// that also reads `.gitignore` files *above* a git worktree. Cursor worktrees
/// live under `~/.cursor/worktrees`, whose parent `.gitignore` is `*`, so every
/// fallback walk would return no files. When `.git` exists as a file or
/// directory at `root` or an ancestor, require a git root so the ignore crate
/// stops there. Otherwise keep local gitignore matching and disable parent
/// search.
pub fn ignore_walk_builder(root: &Path) -> WalkBuilder {
    let mut builder = WalkBuilder::new(root);
    if has_git_metadata(root) {
        builder.require_git(true);
    } else {
        builder.require_git(false).parents(false);
    }
    builder
}

fn has_git_metadata(start: &Path) -> bool {
    start.ancestors().any(|dir| dir.join(".git").exists())
}
