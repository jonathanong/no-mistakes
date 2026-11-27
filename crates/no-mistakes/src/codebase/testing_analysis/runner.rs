#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestRunner {
    Dotnet,
    Playwright,
    Vitest,
    Swift,
    Python,
    Go,
    Cargo,
    Rails,
    Php,
    Java,
    Kotlin,
    Elixir,
    Dart,
    Jest,
}

impl TestRunner {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "dotnet" => Some(Self::Dotnet),
            "playwright" => Some(Self::Playwright),
            "vitest" => Some(Self::Vitest),
            "swift" => Some(Self::Swift),
            "python" => Some(Self::Python),
            "go" => Some(Self::Go),
            "cargo" => Some(Self::Cargo),
            "rails" => Some(Self::Rails),
            "php" => Some(Self::Php),
            "java" => Some(Self::Java),
            "kotlin" => Some(Self::Kotlin),
            "elixir" => Some(Self::Elixir),
            "dart" => Some(Self::Dart),
            "jest" => Some(Self::Jest),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dotnet => "dotnet",
            Self::Playwright => "playwright",
            Self::Vitest => "vitest",
            Self::Swift => "swift",
            Self::Python => "python",
            Self::Go => "go",
            Self::Cargo => "cargo",
            Self::Rails => "rails",
            Self::Php => "php",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Elixir => "elixir",
            Self::Dart => "dart",
            Self::Jest => "jest",
        }
    }

    pub(crate) fn is_language_frontend(self) -> bool {
        matches!(
            self,
            Self::Python
                | Self::Go
                | Self::Cargo
                | Self::Rails
                | Self::Php
                | Self::Java
                | Self::Kotlin
                | Self::Elixir
                | Self::Dart
        )
    }
}
