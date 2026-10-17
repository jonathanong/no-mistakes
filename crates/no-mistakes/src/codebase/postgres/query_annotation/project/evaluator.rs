use super::super::evaluate::{Evaluator, File};
use crate::fx::FxHashMap;
use std::path::PathBuf;

pub(super) fn new<'a, F>(
    files: &'a FxHashMap<PathBuf, File<'a>>,
    resolve: F,
    untrusted_array_intrinsics: bool,
) -> Evaluator<'a, F> {
    super::super::evaluate::new(files, resolve, untrusted_array_intrinsics)
}
