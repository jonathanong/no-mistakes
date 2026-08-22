pub(super) use super::origin::find_target_export_origin;
use super::origin::{origin_for_export, resolve_export_source};
use super::{ExportBucket, ExportOccurrence, ExportOrigin, SourceFile, RULE_ID};
use crate::codebase::symbols::export_kind_str;
use crate::codebase::ts_resolver::{normalize_path, ImportResolverFacade};
use crate::codebase::ts_source::{has_disable_comment, has_disable_line_comment};
use crate::codebase::ts_symbols::{Export, ExportKind};
use crate::codebase::workspaces::WorkspaceMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn collect_file_exports<R: ImportResolverFacade>(
    path: &Path,
    files: &HashMap<PathBuf, SourceFile>,
    resolver: &R,
    workspace: &WorkspaceMap,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    visiting: &mut HashSet<PathBuf>,
    memo: &mut HashMap<PathBuf, Vec<ExportOccurrence>>,
) -> Vec<ExportOccurrence> {
    let path = normalize_path(path);
    if let Some(cached) = memo.get(&path) {
        return cached.clone();
    }
    if !visiting.insert(path.clone()) {
        return Vec::new();
    }
    let Some(file) = files.get(&path) else {
        visiting.remove(&path);
        let out = Vec::new();
        memo.insert(path, out.clone());
        return out;
    };
    if file.disabled && !file.defer_suppression {
        visiting.remove(&path);
        let out = Vec::new();
        memo.insert(path, out.clone());
        return out;
    }
    let mut out = Vec::new();
    for export in &file.symbols.exports {
        if should_skip_export(file, export) {
            continue;
        }
        match &export.kind {
            ExportKind::Default => {}
            ExportKind::ReExport { source, imported } if export.name == "*" && imported == "*" => {
                let Some(target) =
                    resolve_export_source(source, &file.path, resolver, workspace, remapper)
                else {
                    continue;
                };
                for mut occurrence in collect_file_exports(
                    &target, files, resolver, workspace, remapper, visiting, memo,
                ) {
                    if export.is_type_only {
                        if occurrence.bucket == ExportBucket::Value {
                            continue;
                        }
                        occurrence.bucket = ExportBucket::Type;
                    }
                    occurrence.file = file.rel.clone();
                    occurrence.line = export.line;
                    occurrence.kind = export_kind_str(&export.kind).to_string();
                    if let Some(location) = current_suppression_location(file, export) {
                        occurrence.suppression_location = Some(location);
                        occurrence.suppressed = true;
                    }
                    if !skips_framework_export(file, &occurrence.file, &occurrence.name) {
                        out.push(occurrence);
                    }
                }
            }
            ExportKind::ReExport { source, imported } => {
                let resolved =
                    resolve_export_source(source, &file.path, resolver, workspace, remapper);
                let resolved_origin = resolved.as_ref().and_then(|target| {
                    find_target_export_origin(
                        target, imported, files, resolver, workspace, remapper, visiting,
                    )
                });
                let bucket = if export.is_type_only {
                    ExportBucket::Type
                } else if imported == "*" {
                    ExportBucket::Value
                } else {
                    resolved_origin
                        .as_ref()
                        .map(|origin| origin.bucket)
                        .unwrap_or_else(|| ExportBucket::from_export(export))
                };
                let origin_suppressed = resolved_origin
                    .as_ref()
                    .is_some_and(|origin| origin.suppressed);
                let current_suppression = current_suppression_location(file, export);
                let current_suppressed = current_suppression.is_some();
                let suppression_location = current_suppression
                    .or_else(|| suppressed_origin_location(resolved_origin.as_ref()));
                let origin = resolved_origin
                    .map(|origin| {
                        if export.is_type_only {
                            ExportOrigin {
                                bucket: ExportBucket::Type,
                                ..origin
                            }
                        } else {
                            origin
                        }
                    })
                    .unwrap_or_else(|| origin_for_export(file, export, bucket));
                out.push(ExportOccurrence {
                    name: export.name.clone(),
                    bucket,
                    file: file.rel.clone(),
                    line: export.line,
                    kind: export_kind_str(&export.kind).to_string(),
                    origin,
                    suppressed: current_suppressed || origin_suppressed,
                    suppression_location,
                });
            }
            _ => {
                let bucket = ExportBucket::from_export(export);
                let suppression_location = current_suppression_location(file, export);
                out.push(ExportOccurrence {
                    name: export.name.clone(),
                    bucket,
                    file: file.rel.clone(),
                    line: export.line,
                    kind: export_kind_str(&export.kind).to_string(),
                    origin: origin_for_export(file, export, bucket),
                    suppressed: suppression_location.is_some(),
                    suppression_location,
                });
            }
        }
    }

    visiting.remove(&path);
    memo.insert(path, out.clone());
    out
}

pub(super) fn should_skip_export(file: &SourceFile, export: &Export) -> bool {
    export.name == "default"
        || (!file.defer_suppression && current_suppression_location(file, export).is_some())
        || skips_framework_export(file, &file.rel, &export.name)
}

fn skips_framework_export(file: &SourceFile, rel: &str, name: &str) -> bool {
    super::nextjs::is_framework_export(rel, name, file.is_nextjs_project)
        || super::remix::is_framework_export(name, file.is_remix_route_module)
}

pub(super) fn current_suppression_location(
    file: &SourceFile,
    export: &Export,
) -> Option<(String, u32)> {
    (file.disabled
        || has_disable_comment(&file.source, export.line, RULE_ID)
        || has_disable_line_comment(&file.source, export.line, RULE_ID))
    .then(|| (file.rel.clone(), export.line))
}

fn suppressed_origin_location(origin: Option<&ExportOrigin>) -> Option<(String, u32)> {
    origin
        .filter(|origin| origin.suppressed)
        .and_then(|origin| origin.suppression_location.clone())
}
