//! Annotation index construction, isolated from parsing and evaluation.
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::sync::Arc;

pub struct AnnotationModuleIndexes {
    modules: Vec<Module>,
}

impl AnnotationModuleIndexes {
    pub fn binding_count(&self) -> usize {
        self.modules
            .iter()
            .map(|module| module.imports.len() + module.exports.len())
            .sum()
    }
}

pub fn construct_annotation_module_indexes(facts: &[Arc<TsFileFacts>]) -> AnnotationModuleIndexes {
    AnnotationModuleIndexes {
        modules: facts
            .iter()
            .map(|facts| Module::new(Arc::clone(facts)))
            .collect(),
    }
}

#[cfg(test)]
mod tests;
