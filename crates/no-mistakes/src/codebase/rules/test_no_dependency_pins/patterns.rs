pub(super) const DEFAULT_PATTERNS: &[(&str, &str)] = &[
    (
        "exact action ref",
        r"(?<!@)\b[\w.-]+/[\w.-]+@(?:v?\d+(?:\.\d+)*|[a-f0-9]{40})(?:\s*#\s*v?\d+(?:\.\d+)*)?\b",
    ),
    (
        "exact tool version",
        r#"\b[A-Z][A-Z0-9_]*_VERSION:\s*['"]?\d+\.\d+(?:\.\d+)?(?:[-+][A-Za-z0-9_.-]+)?\b"#,
    ),
    (
        "versioned release URL",
        r"\breleases/download/v?\d+(?:\.\d+)+(?:[-+][A-Za-z0-9_.-]+)?\b",
    ),
    (
        "versioned release asset",
        r"\b[A-Za-z0-9_.-]+-v\d+(?:\.\d+)+(?:[-+][A-Za-z0-9_.-]+)?-[A-Za-z0-9_.-]+\b",
    ),
    (
        "versioned tool log",
        r"\bRUN v\d+\.\d+\.\d+(?:[-+][A-Za-z0-9_.-]+)?\b",
    ),
    (
        "package.json dependency assertion",
        r#"\b(?:readFileSync|readRepoFile)\(\s*['"]package\.json['"][^;\n]*?\.(?:toContain|toBe|toEqual)\([^;\n]*?\\?["'][@A-Za-z0-9_./-]+\\?["']\s*:\s*\\?["'][~^]\d+\.\d+\.\d+(?:[-+][A-Za-z0-9_.-]+)?"#,
    ),
    (
        "parsed dependency version assertion",
        r#"\b(?:dependencies|devDependencies|optionalDependencies|peerDependencies)(?:\?\.\s*(?:\[\s*["'][@A-Za-z0-9_./-]+["']\s*\]|[A-Za-z_$][A-Za-z0-9_$-]*)|\s*\[\s*["'][@A-Za-z0-9_./-]+["']\s*\]|\.\s*[A-Za-z_$][A-Za-z0-9_$-]*)[^;\n]*?\.(?:toBe|toEqual)\(\s*['"`][~^]?\d+\.\d+\.\d+(?:[-+][A-Za-z0-9_.-]+)?"#,
    ),
];
