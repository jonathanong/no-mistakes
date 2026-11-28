use crate::codebase::ts_source::SourceStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod manifest;
mod targets;
use targets::{parse_targets, ParsedTarget};

pub(super) enum FileScope {
    Skip,
    PortableTest,
    Uncertain(&'static str),
}

pub(super) struct PackageIndex {
    packages: Vec<PackageInfo>,
}

struct PackageInfo {
    root: PathBuf,
    state: PackageState,
}

enum PackageState {
    Uncertain(&'static str),
    Parsed {
        platforms: Platforms,
        targets: Vec<ParsedTarget>,
        targets_uncertain: bool,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Platforms {
    Portable,
    Uncertain,
}

impl PackageIndex {
    pub(super) fn from_files(all_files: &[PathBuf], sources: &SourceStore) -> Self {
        let manifests = all_files
            .iter()
            .filter(|path| path.file_name().and_then(|n| n.to_str()) == Some("Package.swift"))
            .filter_map(|manifest_path| {
                let root = manifest_path.parent()?.to_path_buf();
                Some((root, sources.read_source_path(manifest_path).ok()))
            });
        Self::from_manifests(manifests)
    }

    pub(super) fn from_manifests(
        manifests: impl IntoIterator<Item = (PathBuf, Option<Arc<str>>)>,
    ) -> Self {
        let mut packages: Vec<_> = manifests
            .into_iter()
            .map(|(root, source)| {
                let state = match source {
                    Some(source) => parse_manifest(&root, &source),
                    None => {
                        PackageState::Uncertain("cannot read Package.swift for this Swift file")
                    }
                };
                PackageInfo { root, state }
            })
            .collect();
        packages.sort_by(|a, b| {
            b.root
                .components()
                .count()
                .cmp(&a.root.components().count())
                .then_with(|| a.root.cmp(&b.root))
        });
        Self { packages }
    }

    pub(super) fn classify(&self, path: &Path) -> FileScope {
        if path.file_name().and_then(|n| n.to_str()) == Some("Package.swift") {
            return FileScope::Skip;
        }
        let Some(package) = self.packages.iter().find(|pkg| path.starts_with(&pkg.root)) else {
            return FileScope::Skip;
        };
        let (platforms, targets, targets_uncertain) = match &package.state {
            PackageState::Uncertain(reason) => return FileScope::Uncertain(reason),
            PackageState::Parsed {
                platforms,
                targets,
                targets_uncertain,
            } => (*platforms, targets, *targets_uncertain),
        };
        let matched = targets
            .iter()
            .filter(|target| target.contains(path))
            .max_by_key(|target| target.root.components().count());
        match matched {
            Some(target) if !target.is_test => FileScope::Skip,
            Some(_) if platforms == Platforms::Portable => FileScope::PortableTest,
            Some(_) => {
                FileScope::Uncertain("cannot statically determine platforms in Package.swift")
            }
            None if targets_uncertain => {
                FileScope::Uncertain("cannot statically determine targets in Package.swift")
            }
            None => FileScope::Skip,
        }
    }
}

fn parse_manifest(package_root: &Path, source: &str) -> PackageState {
    if manifest::has_conditional_directive(source) {
        return PackageState::Uncertain(
            "cannot statically determine targets or platforms in a conditional Package.swift",
        );
    }
    let Some((open, close)) = manifest::find_package_call(source) else {
        return PackageState::Uncertain("cannot statically parse Package.swift");
    };
    let body = &source[open + 1..close];
    let platforms = if manifest::mutates_field_outside(source, open, close, "platforms") {
        Platforms::Uncertain
    } else {
        parse_platforms(body)
    };
    let mut targets_uncertain = manifest::mutates_field_outside(source, open, close, "targets");
    let (targets, parsed_uncertain) = parse_targets(package_root, body);
    targets_uncertain |= parsed_uncertain;
    PackageState::Parsed {
        platforms,
        targets,
        targets_uncertain,
    }
}

fn parse_platforms(body: &str) -> Platforms {
    let Some(expr) = manifest::top_level_label_value(body, "platforms") else {
        return Platforms::Portable;
    };
    let Some(items_body) = manifest::array_body(expr) else {
        return Platforms::Uncertain;
    };
    let items = manifest::split_top_level_items(items_body);
    if items.is_empty() {
        return Platforms::Portable;
    }
    for item in items {
        if !is_known_platform(item) {
            return Platforms::Uncertain;
        }
    }
    Platforms::Portable
}

fn is_known_platform(item: &str) -> bool {
    let Some((name, args)) = manifest::call_name_and_args(item) else {
        return false;
    };
    match name {
        "macOS" | "iOS" | "tvOS" | "watchOS" | "visionOS" | "macCatalyst" | "driverKit"
        | "linux" | "windows" | "android" | "openbsd" | "freebsd" | "wasi" => true,
        "custom" => manifest::split_top_level_items(args)
            .into_iter()
            .next()
            .and_then(manifest::static_string_literal)
            .is_some_and(|platform| {
                matches!(
                    platform.to_ascii_lowercase().as_str(),
                    "macos" | "ios" | "tvos" | "watchos" | "visionos" | "maccatalyst"
                        | "driverkit" | "osx" | "xros" | "linux" | "windows" | "android"
                        | "openbsd" | "freebsd" | "wasi" | "cygwin" | "haiku"
                )
            }),
        _ => false,
    }
}
