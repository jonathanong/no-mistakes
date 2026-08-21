struct DiscoveredPath {
    path: PathBuf,
    index_kind: Option<file_inventory::GitIndexKind>,
}

struct DiscoveredPathViews {
    visible: Vec<DiscoveredPath>,
    tracked: Vec<PathBuf>,
}

fn git_ls_paths(root: &Path) -> Option<Vec<PathBuf>> {
    match git_ls_path_views(root) {
        Ok(Some(views)) => Some(views.visible.into_iter().map(|entry| entry.path).collect()),
        Ok(None) | Err(_) => None,
    }
}

fn git_ls_path_views(root: &Path) -> std::io::Result<Option<DiscoveredPathViews>> {
    let mut cmd = Command::new("git");
    cmd.current_dir(root);
    cmd.arg("ls-files").arg("-z").arg("-t").arg("--stage");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE");
    cmd.arg("--cached")
        .arg("--others")
        // `--deleted` tags missing worktree paths as `R` so we can drop them
        // without `lstat` on every tracked regular file.
        .arg("--deleted")
        .arg("--exclude-standard");
    let out = match crate::invocation::command_output(&mut cmd) {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => return Err(error),
        Err(_) => return Ok(None),
    };
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(parse_git_tagged_paths(&out.stdout)))
}

fn parse_git_tagged_paths(output: &[u8]) -> DiscoveredPathViews {
    let mut visible = Vec::new();
    let mut tracked = Vec::new();
    let mut deleted = HashSet::new();
    for record in output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let [tag, b' ', rest @ ..] = record else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        let (path, index_kind) = parse_git_listed_rest(*tag, rest);
        if path.as_os_str().is_empty() {
            continue;
        }
        if *tag == b'R' {
            deleted.insert(path);
            continue;
        }
        visible.push(DiscoveredPath {
            path: path.clone(),
            index_kind,
        });
        if !matches!(*tag, b'?' | b'K') {
            tracked.push(path);
        }
    }
    visible.retain(|entry| !deleted.contains(&entry.path));
    tracked.retain(|path| !deleted.contains(path));
    sort_dedup_discovered(&mut visible);
    tracked.sort();
    tracked.dedup();
    DiscoveredPathViews { visible, tracked }
}

fn parse_git_listed_rest(tag: u8, rest: &[u8]) -> (PathBuf, Option<file_inventory::GitIndexKind>) {
    if !stage_record_tag(tag) {
        return (git_output_path(rest), None);
    }
    parse_stage_path(rest).map_or_else(
        || (git_output_path(rest), None),
        |(index_kind, path)| {
            (
                git_output_path(path),
                // `--deleted` emits `R` only for missing worktree paths, not
                // skip-worktree/`S` entries, so sparse files need a metadata check.
                if tag == b'S' { None } else { index_kind },
            )
        },
    )
}

fn stage_record_tag(tag: u8) -> bool {
    matches!(tag, b'H' | b'S' | b'M' | b'R' | b'C')
}

fn parse_stage_path(rest: &[u8]) -> Option<(Option<file_inventory::GitIndexKind>, &[u8])> {
    let mode_end = rest.iter().position(|&byte| byte == b' ')?;
    let mode = &rest[..mode_end];
    if mode.len() != 6 || !mode.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let after_mode = &rest[mode_end + 1..];
    let object_end = after_mode.iter().position(|&byte| byte == b' ')?;
    let object = &after_mode[..object_end];
    if object.is_empty() || !object.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let after_object = &after_mode[object_end + 1..];
    let tab = after_object.iter().position(|&byte| byte == b'\t')?;
    let stage = &after_object[..tab];
    if stage.is_empty() || !stage.iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some((git_index_kind(mode), &after_object[tab + 1..]))
}

fn git_index_kind(mode: &[u8]) -> Option<file_inventory::GitIndexKind> {
    match mode {
        b"100644" | b"100755" => Some(file_inventory::GitIndexKind::RegularFile),
        b"120000" => Some(file_inventory::GitIndexKind::Symlink),
        _ => None,
    }
}

fn sort_dedup_discovered(visible: &mut Vec<DiscoveredPath>) {
    visible.sort_by(|left, right| left.path.cmp(&right.path));
    visible.dedup_by(|later, earlier| {
        if later.path != earlier.path {
            return false;
        }
        if earlier.index_kind.is_none() {
            earlier.index_kind = later.index_kind;
        }
        true
    });
}

#[cfg(unix)]
fn git_output_path(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(bytes.to_vec()).into()
}

#[cfg(not(unix))]
fn git_output_path(bytes: &[u8]) -> PathBuf {
    String::from_utf8_lossy(bytes).into_owned().into()
}
