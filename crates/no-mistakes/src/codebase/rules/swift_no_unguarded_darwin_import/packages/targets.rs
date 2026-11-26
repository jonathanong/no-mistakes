use super::manifest;
use crate::codebase::ts_resolver::normalize_path;
use std::path::{Path, PathBuf};

pub(super) struct ParsedTarget {
    pub(super) is_test: bool,
    pub(super) root: PathBuf,
    excludes: Vec<PathBuf>,
    sources: Option<Vec<PathBuf>>,
}

impl ParsedTarget {
    pub(super) fn contains(&self, path: &Path) -> bool {
        if !path.starts_with(&self.root) {
            return false;
        }
        if self
            .excludes
            .iter()
            .any(|excluded| path == excluded || path.starts_with(excluded))
        {
            return false;
        }
        self.sources.as_ref().is_none_or(|sources| {
            sources
                .iter()
                .any(|source| path == source || path.starts_with(source))
        })
    }
}

pub(super) fn parse_targets(package_root: &Path, body: &str) -> (Vec<ParsedTarget>, bool) {
    let Some(expr) = manifest::top_level_label_value(body, "targets") else {
        return (Vec::new(), false);
    };
    let Some(items_body) = manifest::array_body(expr) else {
        return (Vec::new(), true);
    };
    let mut targets = Vec::new();
    let mut uncertain = false;
    for item in manifest::split_top_level_items(items_body) {
        match parse_target(package_root, item) {
            TargetParse::Target(target) => targets.push(target),
            TargetParse::SkipBinary => {}
            TargetParse::Uncertain => uncertain = true,
        }
    }
    (targets, uncertain)
}

enum TargetParse {
    Target(ParsedTarget),
    SkipBinary,
    Uncertain,
}

fn parse_target(package_root: &Path, item: &str) -> TargetParse {
    let Some((kind, args)) = manifest::call_name_and_args(item) else {
        return TargetParse::Uncertain;
    };
    let (is_test, default_dir) = match kind {
        "testTarget" => (true, "Tests"),
        "target" | "executableTarget" | "macro" | "systemLibrary" => (false, "Sources"),
        "plugin" => (false, "Plugins"),
        "binaryTarget" => return TargetParse::SkipBinary,
        _ => return TargetParse::Uncertain,
    };
    let Some(name) =
        manifest::top_level_label_value(args, "name").and_then(manifest::static_string_literal)
    else {
        return TargetParse::Uncertain;
    };
    let rel_path = match manifest::top_level_label_value(args, "path") {
        Some(expr) => match manifest::static_string_literal(expr) {
            Some(path) => PathBuf::from(path),
            None => return TargetParse::Uncertain,
        },
        None => PathBuf::from(default_dir).join(&name),
    };
    let root = normalize_path(&package_root.join(rel_path));
    let Some(excludes) = parse_optional_paths(args, "exclude", &root) else {
        return TargetParse::Uncertain;
    };
    let Some(sources) = parse_optional_paths(args, "sources", &root) else {
        return TargetParse::Uncertain;
    };
    TargetParse::Target(ParsedTarget {
        is_test,
        root,
        excludes: excludes.unwrap_or_default(),
        sources,
    })
}

fn parse_optional_paths(args: &str, label: &str, root: &Path) -> Option<Option<Vec<PathBuf>>> {
    match manifest::top_level_label_value(args, label) {
        Some(expr) => manifest::static_string_array(expr).map(|items| {
            Some(
                items
                    .into_iter()
                    .map(|item| normalize_path(&root.join(item)))
                    .collect(),
            )
        }),
        None => Some(None),
    }
}
