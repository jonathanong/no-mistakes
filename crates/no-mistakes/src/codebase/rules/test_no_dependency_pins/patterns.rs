pub(super) const DEFAULT_PATTERNS: &[(&str, &str, bool)] = &[
    (
        "exact action ref",
        r"(?<!@)\b[\w.-]+/[\w.-]+@(?:v?\d+(?:\.\d+)*|[a-f0-9]{40})(?:\s*#\s*v?\d+(?:\.\d+)*)?\b",
        false,
    ),
    (
        "exact tool version",
        r#"\b[A-Z][A-Z0-9_]*_VERSION:\s*['"]?\d+\.\d+(?:\.\d+)?(?:[-+][A-Za-z0-9_.-]+)?\b"#,
        false,
    ),
    (
        "versioned release URL",
        r"\breleases/download/v?\d+(?:\.\d+)+(?:[-+][A-Za-z0-9_.-]+)?\b",
        false,
    ),
    (
        "versioned release asset",
        r"\b[A-Za-z0-9_.-]+-v\d+(?:\.\d+)+(?:[-+][A-Za-z0-9_.-]+)?-[A-Za-z0-9_.-]+\b",
        false,
    ),
    (
        "versioned tool log",
        r"\bRUN v\d+\.\d+\.\d+(?:[-+][A-Za-z0-9_.-]+)?\b",
        false,
    ),
    (
        "package.json dependency assertion",
        r#"\b(?:readFileSync|readRepoFile)\(\s*['"]package\.json['"][^;\n)]*\)(?:\s*\.(?:toString|trim)\(\))*[^.;\n]*?\.(?:toContain|toBe|toEqual|toStrictEqual)\([^;/\n)]*?\\?["'][@A-Za-z0-9_./-]+\\?["']\s*:\s*\\?["'][~^]?\d+(?:\.\d+){0,2}(?:-[A-Za-z0-9_.-]+)?(?:\+[A-Za-z0-9_.-]+)?\\?["']"#,
        true,
    ),
    (
        "parsed dependency version assertion",
        r#"(?:\.\s*(?:dependencies|devDependencies|optionalDependencies|peerDependencies)|\[\s*["'](?:dependencies|devDependencies|optionalDependencies|peerDependencies)["']\s*\])\s*!?\s*(?:\?\.\s*(?:\[\s*(?:["'][@A-Za-z0-9_./-]+["']|[A-Za-z_$][A-Za-z0-9_$]*(?:\s*\.\s*[A-Za-z_$][A-Za-z0-9_$]*)*)\s*\]|[A-Za-z_$][A-Za-z0-9_$-]*)|\s*\[\s*(?:["'][@A-Za-z0-9_./-]+["']|[A-Za-z_$][A-Za-z0-9_$]*(?:\s*\.\s*[A-Za-z_$][A-Za-z0-9_$]*)*)\s*\]|\.\s*[A-Za-z_$][A-Za-z0-9_$-]*)\s*!?(?:\s+as\s+(?:const|[A-Za-z_$][A-Za-z0-9_.$]*))*\s*\)*(?:\s|//[^\n]*|/\*[^*]*\*+(?:[^/*][^*]*\*+)*/)*(?:,(?:\s|//[^\n]*|/\*[^*]*\*+(?:[^/*][^*]*\*+)*/)*(?:(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\{[^{}]*\}),?)?(?:\s|//[^\n]*|/\*[^*]*\*+(?:[^/*][^*]*\*+)*/)*)?\)\s*\.(?:toBe|toEqual|toStrictEqual)\(\s*['"`](?:npm:[@A-Za-z0-9_./-]+@|workspace:)?(?:[~^=]|[<>]=?)?\d+(?:\.\d+){0,2}(?:-[A-Za-z0-9_.-]+)?(?:\+[A-Za-z0-9_.-]+)?(?:(?:[ \t]*\|\|[ \t]*|[ \t]+-[ \t]+|[ \t]+)(?:[~^=]|[<>]=?)?\d+(?:\.\d+){0,2}(?:-[A-Za-z0-9_.-]+)?(?:\+[A-Za-z0-9_.-]+)?)*["'`]"#,
        true,
    ),
    (
        "parsed dependency version assertion",
        r#"(?:(?:\.\s*(?:dependencies|devDependencies|optionalDependencies|peerDependencies)|\[\s*["'](?:dependencies|devDependencies|optionalDependencies|peerDependencies)["']\s*\])\s*!?(?:\s+as\s+(?:const|[A-Za-z_$][A-Za-z0-9_.$]*))*\s*\)*\s*\)\s*\.toHaveProperty\(\s*(?:["'`][@A-Za-z0-9_./-]+["'`]|[A-Za-z_$][A-Za-z0-9_$]*(?:\s*\.\s*[A-Za-z_$][A-Za-z0-9_$]*)*)|\)\s*\.toHaveProperty\(\s*["'`](?:dependencies|devDependencies|optionalDependencies|peerDependencies)\.[@A-Za-z0-9_./-]+["'`])\s*,\s*["'`](?:npm:[@A-Za-z0-9_./-]+@|workspace:)?(?:[~^=]|[<>]=?)?\d+(?:\.\d+){0,2}(?:-[A-Za-z0-9_.-]+)?(?:\+[A-Za-z0-9_.-]+)?(?:(?:[ \t]*\|\|[ \t]*|[ \t]+-[ \t]+|[ \t]+)(?:[~^=]|[<>]=?)?\d+(?:\.\d+){0,2}(?:-[A-Za-z0-9_.-]+)?(?:\+[A-Za-z0-9_.-]+)?)*["'`]"#,
        true,
    ),
];
