#[derive(Debug, Clone, PartialEq)]
pub struct SpawnEdge {
    /// File that contains the spawn call or webServer declaration.
    pub spawner: PathBuf,
    /// Resolved entry file that is launched.
    pub entry: PathBuf,
}

/// Extract all spawn edges from `source` at `file_path`, resolving entry paths
/// relative to `root`.
///
/// Detects:
/// - Playwright `defineConfig({ webServer: [{ command: '<literal>', cwd?: '<literal>' }] })`
/// - `spawn('<cmd>', args?, opts?)`, `execFile('<cmd>', ...)`, `fork('<module>', ...)`
/// - `exec('<shell command>', ...)` with a string-literal shell command
///
/// String-literal commands are tokenized; env-var assignments (`VAR=value`) are
/// stripped, runtime prefixes (`node`, `tsx`, `npx`) are stripped, and the
/// remaining token is resolved as a file path.
///
/// Template literals whose expressions only appear before the file-path token
/// are also accepted — quasis are concatenated (interpolated values replaced with
/// empty string) and tokenized as above.
///
/// Non-literal arguments (dynamic expressions, ternaries, variable references)
/// are silently skipped — the `process-spawn-static` guardrail enforces literal
/// discipline in target codebases.
pub fn extract_spawn_edges(source: &str, file_path: &Path, root: &Path) -> Vec<SpawnEdge> {
    let allocator = Allocator::default();
    let source_type = SourceType::tsx();
    let ret = Parser::new(&allocator, source, source_type).parse();
    extract_spawn_edges_from_program(&ret.program, source, file_path, root)
}

pub fn extract_spawn_edges_from_program<'a>(
    program: &Program<'a>,
    source: &str,
    file_path: &Path,
    root: &Path,
) -> Vec<SpawnEdge> {
    let mut results = Vec::new();

    for stmt in &program.body {
        collect_from_stmt(stmt, source, file_path, root, &mut results);
    }

    results
}
