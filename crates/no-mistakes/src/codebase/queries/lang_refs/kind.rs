use std::path::Path;

pub(super) enum Frontend {
    Python,
    Go,
    Rust,
    Ruby,
    Php,
    Java,
    Kotlin,
    Elixir,
    Dart,
    Swift,
    Dotnet,
}

impl Frontend {
    pub(super) fn label(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Go => "go",
            Self::Rust => "rust",
            Self::Ruby => "rails",
            Self::Php => "php",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Elixir => "elixir",
            Self::Dart => "dart",
            Self::Swift => "swift",
            Self::Dotnet => "dotnet",
        }
    }
}

pub(super) fn frontend(path: &Path) -> Option<Frontend> {
    if path.file_name().and_then(|name| name.to_str()) == Some("Package.swift") {
        return None;
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("py") => Some(Frontend::Python),
        Some("go") => Some(Frontend::Go),
        Some("rs") => Some(Frontend::Rust),
        Some("rb") => Some(Frontend::Ruby),
        Some("php") => Some(Frontend::Php),
        Some("java") => Some(Frontend::Java),
        Some("kt") => Some(Frontend::Kotlin),
        Some("ex" | "exs") => Some(Frontend::Elixir),
        Some("dart") => Some(Frontend::Dart),
        Some("swift") => Some(Frontend::Swift),
        Some("cs") => Some(Frontend::Dotnet),
        _ => None,
    }
}
