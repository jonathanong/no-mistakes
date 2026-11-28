pub mod ast_complexity;
pub mod lcov_probes;
pub mod oxc_reuse;

use anyhow::Result;
use cpd_core::detect::{detect_prepared, merge_gapped_clones, PathFilters, PreparedSource};
use cpd_core::summary::{file_complexity, span_complexity};
use cpd_tokenizer::tokenizer::{tokenize, tokenize_to_detection, Mode, TokenizeOptions};
use lcov::Report;
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpikeRunOptions {
    pub root: PathBuf,
    #[serde(default)]
    pub lcov_files: Vec<PathBuf>,
    #[serde(default)]
    pub complexity_files: Vec<PathBuf>,
    #[serde(default)]
    pub min_tokens: Option<usize>,
    #[serde(default)]
    pub min_lines: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpikeSummary {
    pub core_clones: usize,
    pub finder_clones: Option<usize>,
    pub complexity_files: Vec<FileComplexityResult>,
    pub lcov_merged: LcovMergeResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileComplexityResult {
    pub path: String,
    pub cpd_file_complexity: u64,
    pub span_complexity_sum: u64,
    pub span_complexity_max: u64,
    pub ast_complexity_sum: u64,
    pub ast_complexity_max: u64,
    pub function_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LcovMergeResult {
    pub sections: usize,
    pub functions_found: usize,
    pub functions_hit: usize,
    pub branches_found: usize,
    pub branches_hit: usize,
    pub lines_found: usize,
    pub lines_hit: usize,
    pub line_hit_sum: u64,
    pub strict_merge_errors: Vec<String>,
}

pub fn format_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("ts" | "mts" | "cts") => "typescript",
        Some("tsx") => "tsx",
        Some("jsx") => "jsx",
        Some("rs") => "rust",
        _ => "javascript",
    }
}

pub fn detect_clones_core_only(
    files: &[(String, String, String)],
    min_tokens: usize,
    min_lines: usize,
) -> usize {
    let opts = TokenizeOptions::new(Mode::Mild);
    let mut by_format: BTreeMap<String, Vec<PreparedSource>> = BTreeMap::new();
    for (id, format, source) in files {
        let tokens = tokenize_to_detection(format, source, &opts);
        if tokens.len() >= min_tokens {
            let prepared =
                PreparedSource::from_detection_tokens(id.clone(), format.clone(), &tokens);
            by_format.entry(format.clone()).or_default().push(prepared);
        }
    }
    let groups: Vec<Vec<PreparedSource>> = by_format.into_values().collect();
    let clones = detect_prepared(groups, min_tokens, min_lines, &PathFilters::default());
    merge_gapped_clones(clones, 0).len()
}

#[cfg(feature = "spike-cpd-finder")]
pub fn detect_clones_finder(root: &Path, min_tokens: usize, min_lines: usize) -> usize {
    let config = cpd_finder::orchestrate::RunConfig {
        paths: vec![root.to_path_buf()],
        min_tokens,
        min_lines,
        similarity: Some(0.8),
        ..cpd_finder::orchestrate::RunConfig::default()
    };
    cpd_finder::orchestrate::run(&config)
        .map(|res| res.clones.len())
        .unwrap_or(0)
}

pub fn measure_file_complexity(path: &Path, source: &str) -> FileComplexityResult {
    let format = format_for_path(path);
    let allocator = Allocator::new();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::tsx());
    let parsed = no_mistakes_ts_facts::ast::parse(path, &allocator, source, source_type);
    let functions = oxc_reuse::collect_functions(&parsed.program, source);
    let cpd_tokens = tokenize(format, source, Mode::Mild);
    let cpd_file = file_complexity(&cpd_tokens, format);
    let mut span_sum = 0u64;
    let mut span_max = 0u64;
    let mut ast_sum = 0u64;
    let mut ast_max = 0u64;
    for func in &functions {
        let sc = span_complexity(&cpd_tokens, format, func.start, func.end);
        span_sum += sc;
        span_max = span_max.max(sc);
        ast_sum += func.ast_complexity;
        ast_max = ast_max.max(func.ast_complexity);
    }
    FileComplexityResult {
        path: path.display().to_string(),
        cpd_file_complexity: cpd_file,
        span_complexity_sum: span_sum,
        span_complexity_max: span_max,
        ast_complexity_sum: ast_sum,
        ast_complexity_max: ast_max,
        function_count: functions.len(),
    }
}

pub fn parse_and_merge_lcov(paths: &[PathBuf]) -> Result<LcovMergeResult> {
    let mut merged = Report::new();
    let mut strict_merge_errors = Vec::new();
    for path in paths {
        let report = Report::from_file(path)?;
        if let Err(err) = merged.merge(report.clone()) {
            strict_merge_errors.push(format!("{}: {err}", path.display()));
            merged.merge_lossy(report);
        }
    }
    let mut out = LcovMergeResult {
        sections: merged.sections.len(),
        functions_found: 0,
        functions_hit: 0,
        branches_found: 0,
        branches_hit: 0,
        lines_found: 0,
        lines_hit: 0,
        line_hit_sum: 0,
        strict_merge_errors,
    };
    for s in merged.sections.values() {
        out.functions_found += s.functions.len();
        out.functions_hit += s.functions.values().filter(|v| v.count > 0).count();
        out.branches_found += s.branches.len();
        out.branches_hit += s
            .branches
            .values()
            .filter(|v| v.taken.unwrap_or(0) > 0)
            .count();
        out.lines_found += s.lines.len();
        out.lines_hit += s.lines.values().filter(|v| v.count > 0).count();
        out.line_hit_sum += s.lines.values().map(|v| v.count).sum::<u64>();
    }
    Ok(out)
}

pub fn run_spike(options: &SpikeRunOptions) -> Result<SpikeSummary> {
    let min_tokens = options.min_tokens.unwrap_or(50);
    let min_lines = options.min_lines.unwrap_or(5);
    let mut inputs = Vec::new();
    let mut complexity_files = Vec::new();
    for path in &options.complexity_files {
        let source = std::fs::read_to_string(path)?;
        complexity_files.push(measure_file_complexity(path, &source));
        inputs.push((
            path.display().to_string(),
            format_for_path(path).to_string(),
            source,
        ));
    }
    let core_clones = detect_clones_core_only(&inputs, min_tokens, min_lines);
    #[cfg(feature = "spike-cpd-finder")]
    let finder_clones = Some(detect_clones_finder(&options.root, min_tokens, min_lines));
    #[cfg(not(feature = "spike-cpd-finder"))]
    let finder_clones = None;
    let lcov_merged = parse_and_merge_lcov(&options.lcov_files)?;
    Ok(SpikeSummary {
        core_clones,
        finder_clones,
        complexity_files,
        lcov_merged,
    })
}
