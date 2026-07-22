use super::{ExportBucket, ExportOrigin, SourceFile};
use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::codebase::ts_symbols::{Export, ExportKind};
use crate::codebase::workspaces::WorkspaceMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn find_target_export_origin<R: ImportResolverFacade>(
    target: &Path,
    imported: &str,
    files: &HashMap<PathBuf, SourceFile>,
    resolver: &R,
    workspace: &WorkspaceMap,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    visiting: &mut HashSet<PathBuf>,
) -> Option<ExportOrigin> {
    OriginSearch {
        files,
        resolver,
        workspace,
        remapper,
        visiting,
    }
    .find(target, imported)
}

struct OriginSearch<'a, R: ImportResolverFacade> {
    files: &'a HashMap<PathBuf, SourceFile>,
    resolver: &'a R,
    workspace: &'a WorkspaceMap,
    remapper: &'a crate::codebase::ts_source::FrozenPathRemapper,
    visiting: &'a mut HashSet<PathBuf>,
}

impl<R: ImportResolverFacade> OriginSearch<'_, R> {
    fn find(&mut self, target: &Path, imported: &str) -> Option<ExportOrigin> {
        let target = self.remapper.remap(target);
        if !self.visiting.insert(target.clone()) {
            return None;
        }
        let Some(file) = self.files.get(&target) else {
            self.visiting.remove(&target);
            return None;
        };
        if file.disabled {
            self.visiting.remove(&target);
            return None;
        }

        let found = file
            .symbols
            .exports
            .iter()
            .filter(|export| !super::collector::should_skip_export(file, export))
            .find_map(|export| self.find_export(file, export, imported));
        self.visiting.remove(&target);
        found
    }

    fn find_export(
        &mut self,
        file: &SourceFile,
        export: &Export,
        imported: &str,
    ) -> Option<ExportOrigin> {
        match &export.kind {
            ExportKind::Default if imported == "default" => Some(origin_for_export(
                file,
                export,
                ExportBucket::from_export(export),
            )),
            ExportKind::ReExport {
                source,
                imported: reimported,
            } if export.name == imported => {
                self.explicit_reexport_origin(file, export, source, reimported)
            }
            ExportKind::ReExport {
                source,
                imported: reimported,
            } if export.name == "*" && reimported == "*" => resolve_export_source(
                source,
                &file.path,
                self.resolver,
                self.workspace,
                self.remapper,
            )
            .and_then(|resolved| self.find(&resolved, imported)),
            _ if export.name == imported => Some(origin_for_export(
                file,
                export,
                ExportBucket::from_export(export),
            )),
            _ => None,
        }
    }

    fn explicit_reexport_origin(
        &mut self,
        file: &SourceFile,
        export: &Export,
        source: &str,
        reimported: &str,
    ) -> Option<ExportOrigin> {
        let resolved_origin = match resolve_export_source(
            source,
            &file.path,
            self.resolver,
            self.workspace,
            self.remapper,
        ) {
            Some(resolved) => self.find(&resolved, reimported),
            None => None,
        };
        if export.is_type_only {
            if let Some(origin) = resolved_origin {
                Some(ExportOrigin {
                    bucket: ExportBucket::Type,
                    ..origin
                })
            } else {
                Some(origin_for_export(file, export, ExportBucket::Type))
            }
        } else {
            if resolved_origin.is_some() {
                resolved_origin
            } else {
                Some(origin_for_export(file, export, ExportBucket::Value))
            }
        }
    }
}

pub(super) fn origin_for_export(
    file: &SourceFile,
    export: &Export,
    bucket: ExportBucket,
) -> ExportOrigin {
    ExportOrigin {
        file: file.rel.clone(),
        line: export.line,
        name: export.name.clone(),
        bucket,
    }
}

pub(super) fn resolve_export_source<R: ImportResolverFacade>(
    source: &str,
    importing_file: &Path,
    resolver: &R,
    workspace: &WorkspaceMap,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Option<PathBuf> {
    if let Some(path) = resolver.resolve(source, importing_file) {
        return Some(remapper.remap(&path));
    }
    let workspace_path = match resolver.visible_files() {
        Some(visible) => {
            workspace.resolve_specifier_from_file_visible(source, importing_file, visible)
        }
        None => workspace.resolve_specifier(source),
    };
    if let Some(path) = workspace_path {
        return Some(remapper.remap(&path));
    }
    None
}
