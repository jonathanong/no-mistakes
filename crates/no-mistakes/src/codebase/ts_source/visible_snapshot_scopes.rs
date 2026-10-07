/// Reuse one prepared view for a configured Git boundary, rather than
/// rediscovering the same index for each sibling project. Never ascend into a
/// shared outer ancestor: an unrelated enclosing repository is not an opt-in
/// extension of the configured scope.
fn snapshot_scope_root(request_root: &Path, root: &Path) -> PathBuf {
    let boundary = root
        .ancestors()
        .take_while(|ancestor| !request_root.starts_with(ancestor))
        .find(|ancestor| ancestor.join(".git").exists());
    boundary.map_or_else(
        || {
            if root.starts_with(request_root) {
                request_root.to_path_buf()
            } else {
                root.to_path_buf()
            }
        },
        Path::to_path_buf,
    )
}

impl VisiblePathSnapshot {
    fn cached_scope_root(&self, root: &Path) -> PathBuf {
        let mut scopes = self
            .scope_roots
            .lock()
            .expect("snapshot scope mutex poisoned");
        scopes
            .entry(root.to_path_buf())
            .or_insert_with(|| snapshot_scope_root(&self.request_root, root))
            .clone()
    }
}
