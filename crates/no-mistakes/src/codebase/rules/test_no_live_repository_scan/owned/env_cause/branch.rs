use super::diagnostic_paths;
use super::Owners;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{
    ConditionalPlace, EnvWrite, PathProvenance, WriteScope,
};
use std::path::{Path, PathBuf};

pub(super) enum Cover {
    None,
    One(u32),
    Many,
}

pub(super) enum FileHit {
    Absent,
    Branch(u32),
    Blocked,
}

impl Owners<'_> {
    pub(super) fn conditional_branch(
        &self,
        file: &Path,
        name: &str,
        offset: Option<u32>,
        facts: &dyn TsFactLookup,
    ) -> Option<(PathBuf, u32)> {
        if let Some(offset) = offset {
            if let Some(paths) = facts.get_ts_facts(file).and_then(diagnostic_paths) {
                match covering_line(paths, name, offset) {
                    Cover::One(line) => return Some((file.to_path_buf(), line)),
                    Cover::Many => return None,
                    Cover::None => {}
                }
            }
        }
        external_branch(&self.related_files(file), name, facts)
    }

    fn related_files(&self, file: &Path) -> Vec<PathBuf> {
        let mut files = self.setups.setup_files_for(file);
        for (node, kind) in self
            .graph
            .dependencies_of_node(&NodeId::file(file))
            .into_iter()
            .flatten()
        {
            if *kind == EdgeKind::Import {
                if let Some(path) = node.as_file() {
                    if path != file {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
        files.retain(|path| path != file);
        files.sort();
        files.dedup();
        files
    }
}

pub(super) fn covering_line(paths: &PathProvenance, name: &str, offset: u32) -> Cover {
    let mut found = Cover::None;
    for write in &paths.env_writes {
        if write.name != name || !covers(write, paths, offset) {
            continue;
        }
        found = match found {
            Cover::None => Cover::One(write.line),
            Cover::One(_) | Cover::Many => Cover::Many,
        };
    }
    found
}

fn covers(write: &EnvWrite, paths: &PathProvenance, offset: u32) -> bool {
    let WriteScope::Conditional(place) = write.scope else {
        return false;
    };
    if !place.branch {
        return false;
    }
    if let Some((start, end)) = place.hook {
        return paths
            .test_callbacks
            .iter()
            .any(|(from, to)| start <= *from && *to <= end && *from <= offset && offset < *to);
    }
    if let Some((start, end)) = place.function {
        return start <= offset && offset < end && write.end <= offset;
    }
    place.program && write.end <= offset
}

pub(super) fn file_hit(paths: &PathProvenance, name: &str) -> FileHit {
    let mut line = None;
    for write in paths.env_writes.iter().filter(|write| write.name == name) {
        let WriteScope::Conditional(ConditionalPlace { branch: true, .. }) = write.scope else {
            return FileHit::Blocked;
        };
        if line.is_some() {
            return FileHit::Blocked;
        }
        line = Some(write.line);
    }
    match line {
        Some(line) => FileHit::Branch(line),
        None => FileHit::Absent,
    }
}

pub(super) fn external_branch(
    files: &[PathBuf],
    name: &str,
    facts: &dyn TsFactLookup,
) -> Option<(PathBuf, u32)> {
    let mut found = None;
    for file in files {
        let Some(paths) = facts.get_ts_facts(file).and_then(diagnostic_paths) else {
            continue;
        };
        match file_hit(paths, name) {
            FileHit::Absent => {}
            FileHit::Blocked => return None,
            FileHit::Branch(line) => {
                if found.is_some() {
                    return None;
                }
                found = Some((file.clone(), line));
            }
        }
    }
    found
}
