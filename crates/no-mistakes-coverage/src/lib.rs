//! LCOV coverage parsing, merging, path normalization, and deterministic serialization.

pub mod merge;
pub mod model;
pub mod normalize;
pub mod parse;
pub mod serialize;

pub use merge::{merge_file_coverage, merge_into, merge_reports};
pub use model::{
    BranchKey, CoverageReport, FileCoverage, FunctionCoverage, ParseDiagnostic,
    ParseDiagnosticKind, ParsedLcov,
};
pub use normalize::{normalize_path, NormalizeOptions, PathNormalizeError};
pub use parse::{parse_lcov_bytes, parse_lcov_str, ParseOptions};
pub use serialize::{to_lcov, to_lcov_minimal};
