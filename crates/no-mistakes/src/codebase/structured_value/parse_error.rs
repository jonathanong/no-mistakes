use super::{merge_error, merge_keys::MergeError, PartialYaml};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct StructuredValueError {
    pub(crate) line: Option<usize>,
    pub(crate) message: String,
    pub(crate) missing_merge_location: bool,
    pub(crate) partial: Option<Arc<PartialYaml>>,
}

impl StructuredValueError {
    pub(super) fn parse(message: String) -> Self {
        Self {
            line: None,
            message,
            missing_merge_location: false,
            partial: None,
        }
    }

    pub(super) fn unlocated(line: usize, partial: PartialYaml) -> Self {
        Self {
            partial: Some(Arc::new(partial)),
            ..Self::merge(MergeError::MissingLocation(line))
        }
    }

    pub(super) fn merge(error: MergeError) -> Self {
        let missing_merge_location = matches!(error, MergeError::MissingLocation(_));
        let (line, detail) = error.detail();
        Self {
            line: Some(line),
            message: merge_error(&detail),
            missing_merge_location,
            partial: None,
        }
    }
}

impl std::fmt::Display for StructuredValueError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(formatter, "{} at line {line}", self.message),
            None => formatter.write_str(&self.message),
        }
    }
}
