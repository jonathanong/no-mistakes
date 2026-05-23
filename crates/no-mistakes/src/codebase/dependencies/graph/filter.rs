pub struct FilterSpec {
    file_set: Option<GlobSet>,
    folder_specs: Vec<FolderSpec>,
}

struct FolderSpec {
    ancestor_depth: usize,
    set: GlobSet,
}

/// Build a `FilterSpec` from patterns.
///
/// Patterns ending with `/` are folder patterns: they collapse matched files to
/// the folder ancestor at that depth.
///
/// Returns `None` if `patterns` is empty (no filter applied).
pub fn build_filter(patterns: &[String]) -> Result<Option<FilterSpec>> {
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut file_builder = GlobSetBuilder::new();
    let mut has_file = false;
    let mut folder_specs: Vec<FolderSpec> = Vec::new();

    for pattern in patterns {
        if let Some(base) = pattern.strip_suffix('/') {
            let ancestor_depth = Path::new(base).components().count();
            let file_glob = format!("{base}/**");
            let mut builder = GlobSetBuilder::new();
            builder.add(Glob::new(&file_glob)?);
            folder_specs.push(FolderSpec {
                ancestor_depth,
                set: builder.build()?,
            });
        } else {
            file_builder.add(Glob::new(pattern)?);
            has_file = true;
        }
    }

    Ok(Some(FilterSpec {
        file_set: if has_file {
            Some(file_builder.build()?)
        } else {
            None
        },
        folder_specs,
    }))
}

/// Retain only entries matching `filter`.
/// QueueJob virtual nodes pass through without path-based filtering.
pub fn apply_filter(
    entries: Vec<NodeEntry>,
    filter: Option<&FilterSpec>,
    root: &Path,
) -> Vec<NodeEntry> {
    let filter = match filter {
        None => return entries,
        Some(f) => f,
    };

    let mut result: Vec<NodeEntry> = Vec::new();
    let mut folder_seen: HashMap<PathBuf, usize> = HashMap::new();

    for entry in entries {
        // Virtual nodes (QueueJob) pass through without file-path filtering.
        let file_path = match entry.node.as_file() {
            Some(p) => p,
            None => {
                result.push(entry);
                continue;
            }
        };

        let rel = file_path.strip_prefix(root).unwrap_or(file_path);

        let mut matched_folder = false;
        for spec in &filter.folder_specs {
            if spec.set.is_match(rel) {
                let folder: PathBuf = rel.components().take(spec.ancestor_depth).collect();
                if let Some(&idx) = folder_seen.get(&folder) {
                    if entry.depth < result[idx].depth {
                        result[idx].depth = entry.depth;
                    }
                } else {
                    let idx = result.len();
                    folder_seen.insert(folder.clone(), idx);
                    result.push(NodeEntry {
                        node: NodeId::File(root.join(&folder)),
                        depth: entry.depth,
                        via: entry.via.clone(),
                    });
                }
                matched_folder = true;
                break;
            }
        }
        if matched_folder {
            continue;
        }

        if let Some(gs) = &filter.file_set {
            if gs.is_match(rel) {
                result.push(entry);
            }
        }
    }

    result
}

// ── Symbol index ─────────────────────────────────────────────────────────────
