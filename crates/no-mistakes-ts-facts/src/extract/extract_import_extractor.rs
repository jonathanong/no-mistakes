use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::path::Path;

/// Holds parser configuration for TypeScript or TSX extraction.
pub struct ImportExtractor {
    is_tsx: bool,
}

impl ImportExtractor {
    pub fn for_typescript() -> Result<Self> {
        Ok(Self { is_tsx: false })
    }

    pub fn for_tsx() -> Result<Self> {
        Ok(Self { is_tsx: true })
    }

    /// Extract import/export specifier strings from `source`, tagging each
    /// with the syntax form that created the dependency.
    pub fn extract(&self, source: &str) -> Result<Vec<ExtractedImport>> {
        let allocator = Allocator::default();
        let source_type = if self.is_tsx {
            SourceType::tsx()
        } else {
            SourceType::ts()
        };
        let sentinel = if self.is_tsx {
            "imports.tsx"
        } else {
            "imports.ts"
        };
        let ret = crate::ast::parse(Path::new(sentinel), &allocator, source, source_type);

        Ok(
            extract_import_facts_from_program_with_source_and_resource_roots(
                &ret.program,
                source,
                false,
            )
            .imports,
        )
    }
}
