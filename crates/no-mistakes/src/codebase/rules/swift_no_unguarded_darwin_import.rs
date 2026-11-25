use super::masked_source::FileAllow;
use super::{MaskedFileContext, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::sync::LazyLock;

pub const RULE_ID: &str = "swift-no-unguarded-darwin-import";

const FIX: &str =
    "Wrap the Darwin import and the tests that use it in `#if canImport(Darwin)`.";

static OPEN_CONDITIONAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|;)[ \t\x0B\x00]*#if(?:[ \t\x0B\x00(!]|$)").unwrap()
});

static DARWIN_IMPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"^(?:@[A-Za-z_][A-Za-z0-9_]*[ \t]+)*",
        r"(?:(?:public|package|internal|fileprivate|private)[ \t]+)?",
        r"import[ \t]+(?:(?:typealias|struct|class|enum|protocol|let|var|func)[ \t]+)?",
        r"(?:Darwin|\x60Darwin\x60)[ \t]*(?:[.;]|$)",
    ))
    .unwrap()
});

#[derive(Deserialize, Default)]
#[serde(default)]
struct Options {
    allow: Vec<String>,
    message: Option<String>,
}

struct CompiledOptions {
    allow: FileAllow,
    message: String,
}

super::masked_source_rule_entrypoints!(compile_options, is_swift_file, check_file);

fn compile_options(options: Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        allow: FileAllow::new(&options.allow, &format!("{RULE_ID} allow"))?,
        message: match options.message.filter(|message| !message.is_empty()) {
            Some(message) => format!("{FIX} {message}"),
            None => FIX.to_string(),
        },
    })
}

fn is_swift_file(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("swift")
}

fn check_file(
    root: &Path,
    path: &Path,
    options: &CompiledOptions,
    sources: &SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    super::scan_masked_file(
        MaskedFileContext::new(root, path, RULE_ID, &options.allow, sources, defer_suppression),
        super::lexical_mask::swift_code_mask,
        |_, code, file| {
            import_lines(code)
                .map(|line| RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: file.to_string(),
                    line,
                    message: options.message.clone(),
                    import: None,
                    target: None,
                    causes: Vec::new(),
                })
                .collect()
        },
    )
}

fn import_lines(code: &str) -> impl Iterator<Item = usize> + '_ {
    let mut conditional_depth = 0usize;
    code.lines().enumerate().filter_map(move |(index, line)| {
        let line = line.trim_matches([' ', '\t', '\x0B', '\0']);
        if matches!(line, "#endif" | "#endif;") {
            conditional_depth = conditional_depth.saturating_sub(1);
        } else if OPEN_CONDITIONAL.is_match(line) {
            conditional_depth += 1;
        } else if conditional_depth == 0 && DARWIN_IMPORT.is_match(line) {
            return Some(index + 1);
        }
        None
    })
}
