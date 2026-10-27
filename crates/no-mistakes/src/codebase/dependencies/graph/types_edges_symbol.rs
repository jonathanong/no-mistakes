impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        self.as_symbol_str()
            .or_else(|| self.as_core_str())
            .unwrap_or_else(|| self.as_domain_str())
    }

    fn as_symbol_str(&self) -> Option<&'static str> {
        match self {
            Self::SymbolImport => Some("symbol-import"),
            Self::SymbolReexport => Some("symbol-reexport"),
            Self::SymbolImportUnknown => Some("symbol-import-unknown"),
            Self::SymbolTypeImport => Some("symbol-type-import"),
            Self::SymbolTypeReexport => Some("symbol-type-reexport"),
            _ => None,
        }
    }
}
