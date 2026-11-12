use super::{Import, Pending, Signature};
use crate::codebase::check_facts::{
    sql_source_prerequisites::{source_failed, FailedSqlSources},
    CheckFileFacts,
};
use crate::codebase::ts_source::FileIdMap;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
struct Proof {
    signature: Option<Signature>,
    failed: FailedSqlSources,
}

pub(in super::super) struct Resolver<'a, F> {
    files: &'a FileIdMap<CheckFileFacts>,
    resolve: &'a F,
    memo: FxHashMap<((PathBuf, String), usize), Proof>,
    active: FxHashSet<(PathBuf, String)>,
}

impl<'a, F: Fn(&str, &Path) -> Option<PathBuf>> Resolver<'a, F> {
    pub fn new(files: &'a FileIdMap<CheckFileFacts>, resolve: &'a F) -> Self {
        Self {
            files,
            resolve,
            memo: Default::default(),
            active: Default::default(),
        }
    }
    pub fn project(&mut self, path: &Path, pending: &Pending) -> (bool, FailedSqlSources) {
        self.requirements(path, &pending.required)
    }
    fn requirements(&mut self, path: &Path, required: &[Import]) -> (bool, FailedSqlSources) {
        let mut proven = true;
        let mut failed = FailedSqlSources::default();
        for import in required {
            let Some(target) = (self.resolve)(&import.specifier, path) else {
                proven = false;
                continue;
            };
            let proof = self.lookup(&target, &import.imported);
            failed.extend(proof.failed);
            proven &= proof
                .signature
                .is_some_and(|signature| signature.accepts(import.shape));
        }
        (proven, failed)
    }
    fn lookup(&mut self, path: &Path, name: &str) -> Proof {
        let identity = (path.to_path_buf(), name.to_string());
        let key = (identity.clone(), self.active.len());
        if let Some(proof) = self.memo.get(&key) {
            return proof.clone();
        }
        if self.active.len() >= 16 || !self.active.insert(identity.clone()) {
            return Proof::default();
        }
        let proof = self.compute(path, name);
        self.active.remove(&identity);
        self.memo.insert(key, proof.clone());
        proof
    }
    fn compute(&mut self, path: &Path, name: &str) -> Proof {
        if source_failed(self.files, path) {
            return Proof {
                signature: None,
                failed: [path.to_path_buf()].into_iter().collect(),
            };
        }
        let files = self.files;
        let Some(file) = files.get(path) else {
            return Proof::default();
        };
        if let Some(heads) = file.query_annotation_heads.as_deref() {
            if let Some(&signature) = heads.exported_effects.get(name) {
                return Proof {
                    signature: Some(signature),
                    ..Default::default()
                };
            }
            if let Some(summary) = heads.effect_summaries.get(name) {
                let (proven, failed) = self.requirements(path, &summary.required);
                return Proof {
                    signature: proven.then_some(summary.signature),
                    failed,
                };
            }
        }
        let Some(module) = file.ts.module_bindings.as_deref() else {
            return Proof::default();
        };
        let mut exports = module
            .exports
            .iter()
            .filter(|export| !export.type_only && export.exported == name);
        let Some(export) = exports.next() else {
            return Proof::default();
        };
        if exports.next().is_some() {
            return Proof::default();
        }
        let Some(specifier) = &export.specifier else {
            return Proof::default();
        };
        let Some(target) = (self.resolve)(specifier, path) else {
            return Proof::default();
        };
        let imported = export.local.clone();
        self.lookup(&target, &imported)
    }
}
