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

pub(super) enum Candidate {
    Absent,
    One(PathBuf, u32),
    Abstain,
}

impl Owners<'_> {
    pub(super) fn conditional_branch(
        &self,
        file: &Path,
        name: &str,
        offset: Option<u32>,
        facts: &dyn TsFactLookup,
    ) -> Option<(PathBuf, u32)> {
        let local = local_candidate(file, name, offset, facts);
        let external = external_candidate(&self.related_files(file), name, facts);
        match (local, external) {
            (Candidate::One(path, line), Candidate::Absent)
            | (Candidate::Absent, Candidate::One(path, line)) => Some((path, line)),
            _ => None,
        }
    }

    fn related_files(&self, file: &Path) -> Vec<PathBuf> {
        let mut files = self.setups.setup_files_for(file);
        let mut pending: Vec<NodeId> = files.iter().map(|path| NodeId::file(path)).collect();
        pending.push(NodeId::file(file));
        let mut seen = pending.clone();
        while let Some(current) = pending.pop() {
            for (node, kind) in self
                .graph
                .dependencies_of_node(&current)
                .into_iter()
                .flatten()
            {
                if *kind != EdgeKind::Import {
                    continue;
                }
                let NodeId::File(path) = node else {
                    continue;
                };
                if seen.iter().any(|seen| seen == node) {
                    continue;
                }
                seen.push(node.clone());
                pending.push(node.clone());
                if path.as_ref() != file {
                    files.push(path.as_ref().to_path_buf());
                }
            }
        }
        files.retain(|path| path != file);
        files.sort();
        files.dedup();
        files
    }
}

fn local_candidate(
    file: &Path,
    name: &str,
    offset: Option<u32>,
    facts: &dyn TsFactLookup,
) -> Candidate {
    let Some(offset) = offset else {
        return Candidate::Absent;
    };
    let Some(paths) = facts.get_ts_facts(file).and_then(diagnostic_paths) else {
        return Candidate::Absent;
    };
    match covering_line(paths, name, offset) {
        Cover::None => Candidate::Absent,
        Cover::One(line) => Candidate::One(file.to_path_buf(), line),
        Cover::Many => Candidate::Abstain,
    }
}

pub(super) fn covering_line(paths: &PathProvenance, name: &str, offset: u32) -> Cover {
    let mut found = Cover::None;
    for write in &paths.env_writes {
        let WriteScope::Conditional(place) = write.scope else {
            continue;
        };
        if write.name != name || !reaches(place, write, paths, offset) {
            continue;
        }
        if !place.branch {
            return Cover::Many;
        }
        found = match found {
            Cover::None => Cover::One(write.line),
            Cover::One(_) | Cover::Many => Cover::Many,
        };
    }
    found
}

fn reaches(place: ConditionalPlace, write: &EnvWrite, paths: &PathProvenance, offset: u32) -> bool {
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
        let WriteScope::Conditional(ConditionalPlace {
            branch: true,
            function: None,
            ..
        }) = write.scope
        else {
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

pub(super) fn external_candidate(
    files: &[PathBuf],
    name: &str,
    facts: &dyn TsFactLookup,
) -> Candidate {
    let mut found = None;
    for file in files {
        let Some(paths) = facts.get_ts_facts(file).and_then(diagnostic_paths) else {
            return Candidate::Abstain;
        };
        match file_hit(paths, name) {
            FileHit::Absent => {}
            FileHit::Blocked => return Candidate::Abstain,
            FileHit::Branch(line) => {
                if found.is_some() {
                    return Candidate::Abstain;
                }
                found = Some((file.clone(), line));
            }
        }
    }
    match found {
        Some((file, line)) => Candidate::One(file, line),
        None => Candidate::Absent,
    }
}
