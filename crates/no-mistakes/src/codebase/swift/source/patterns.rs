use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

static IMPORT: OnceLock<Regex> = OnceLock::new();
static DECLARATION: OnceLock<Regex> = OnceLock::new();
static FUNCTION: OnceLock<Regex> = OnceLock::new();
static PROPERTY: OnceLock<Regex> = OnceLock::new();
static REFERENCE: OnceLock<Regex> = OnceLock::new();
static ENDPOINT_PATH: OnceLock<Regex> = OnceLock::new();
static INTERPOLATION: OnceLock<Regex> = OnceLock::new();
static REFERENCE_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

pub(super) fn swift_import_regex() -> &'static Regex {
    IMPORT.get_or_init(|| {
        Regex::new(r"(?m)^\s*import\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid import regex")
    })
}

pub(super) fn swift_declaration_regex() -> &'static Regex {
    DECLARATION.get_or_init(|| {
        Regex::new(r"\b(?:public\s+|internal\s+|private\s+|fileprivate\s+|open\s+|final\s+|static\s+|class\s+)*\b(?:struct|class|actor|enum|protocol|extension|typealias)\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid declaration regex")
    })
}

pub(super) fn swift_function_regex() -> &'static Regex {
    FUNCTION.get_or_init(|| {
        Regex::new(r"\b(?:static\s+|class\s+)?func\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid function regex")
    })
}

pub(super) fn swift_property_regex() -> &'static Regex {
    PROPERTY.get_or_init(|| {
        Regex::new(r"\b(?:static\s+|class\s+)?(?:let|var)\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid property regex")
    })
}

pub(super) fn swift_reference_regex() -> &'static Regex {
    REFERENCE.get_or_init(|| {
        Regex::new(r"\b[A-Z_][A-Za-z0-9_]*\b|\.[A-Za-z_][A-Za-z0-9_]*\b")
            .expect("valid reference regex")
    })
}

pub(super) fn swift_endpoint_path_regex() -> &'static Regex {
    ENDPOINT_PATH.get_or_init(|| {
        Regex::new(r#"path\s*:\s*\"([^\"]+)\""#).expect("valid endpoint path regex")
    })
}

pub(super) fn swift_interpolation_regex() -> &'static Regex {
    INTERPOLATION.get_or_init(|| Regex::new(r#"\\\([^)]*\)"#).expect("valid interpolation regex"))
}

pub(super) fn swift_reference_keywords() -> &'static HashSet<&'static str> {
    REFERENCE_KEYWORDS.get_or_init(|| {
        [
            "Array",
            "Bool",
            "Data",
            "Dictionary",
            "Double",
            "Error",
            "False",
            "Float",
            "Int",
            "Nil",
            "Optional",
            "Result",
            "Self",
            "Set",
            "String",
            "True",
            "Void",
        ]
        .into_iter()
        .collect()
    })
}
