#[derive(Debug, Clone, PartialEq)]
pub enum ExportKind {
    Function,
    Class,
    Const,
    Let,
    Var,
    TypeAlias,
    Interface,
    Enum,
    Default,
    /// Re-export: `export { name } from 'source'` or `export * from 'source'`.
    ReExport {
        source: String,
        /// The imported symbol name in the source module. `"*"` for star re-exports.
        imported: String,
    },
}

/// A top-level exported symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct Export {
    /// The public exported name.
    pub name: String,
    /// The local binding that backs this export, when it differs from `name`.
    pub local: Option<String>,
    pub kind: ExportKind,
    pub line: u32,
    pub is_type_only: bool,
}

/// A named import statement.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedImport {
    /// The module specifier (e.g. `"./foo.mts"`, `"@utils/helpers"`).
    pub source: String,
    /// The name as exported from the source module.
    pub imported: String,
    /// The local binding name (may differ from `imported` when aliased).
    pub local: String,
    pub line: u32,
    pub is_type_only: bool,
}

/// All top-level exports and named imports extracted from a file.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FileSymbols {
    pub exports: Vec<Export>,
    pub imports: Vec<NamedImport>,
}

/// Extract top-level exports and named imports from TypeScript/TSX source.
pub fn extract_symbols(source: &str, is_tsx: bool) -> Result<FileSymbols> {
    let allocator = Allocator::default();
    let source_type = if is_tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let sentinel = if is_tsx { "symbols.tsx" } else { "symbols.ts" };
    let ret = crate::ast::parse(
        std::path::Path::new(sentinel),
        &allocator,
        source,
        source_type,
    );
    if ret.panicked {
        let detail = ret
            .diagnostics
            .first()
            .map(|err| format!("{err:?}"))
            .unwrap_or("unknown error (parser panicked)".to_string());
        bail!("failed to parse TypeScript source: {detail}");
    }

    Ok(extract_symbols_from_program(&ret.program, source))
}

pub fn extract_symbols_from_program(program: &Program<'_>, source: &str) -> FileSymbols {
    let mut symbols = FileSymbols::default();
    let local_type_names = local_type_declaration_names(program);
    for stmt in &program.body {
        process_statement(stmt, source, &local_type_names, &mut symbols);
    }
    symbols
}

fn local_type_declaration_names(program: &Program<'_>) -> HashSet<String> {
    program
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::TSTypeAliasDeclaration(decl) => Some(decl.id.name.as_str().to_string()),
            Statement::TSInterfaceDeclaration(decl) => Some(decl.id.name.as_str().to_string()),
            _ => None,
        })
        .collect()
}
