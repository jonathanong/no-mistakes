pub(crate) const VITEST_JEST_TEST_GLOBS: &[&str] = &[
    "**/*.test.mts",
    "**/*.spec.mts",
    "**/*.test.ts",
    "**/*.spec.ts",
    "**/*.test.tsx",
    "**/*.spec.tsx",
    "**/*.test.mjs",
    "**/*.spec.mjs",
    "**/*.test.js",
    "**/*.spec.js",
    "**/*.test.jsx",
    "**/*.spec.jsx",
    "**/__tests__/**/*.mts",
    "**/__tests__/**/*.ts",
    "**/__tests__/**/*.tsx",
    "**/__tests__/**/*.mjs",
    "**/__tests__/**/*.js",
    "**/__tests__/**/*.jsx",
];

/// Map a `--test <framework>` value to its corresponding glob patterns.
pub(crate) fn test_globs(framework: &str) -> Vec<String> {
    const PLAYWRIGHT: &[&str] = &[
        "**/tests/e2e/**/*.mts",
        "**/tests/e2e/**/*.ts",
        "**/tests/e2e/**/*.tsx",
        "**/tests/e2e/**/*.mjs",
        "**/tests/e2e/**/*.js",
        "**/tests/e2e/**/*.jsx",
        "**/playwright/**/*.spec.mts",
        "**/playwright/**/*.spec.ts",
        "**/playwright/**/*.spec.tsx",
        "**/playwright/**/*.spec.mjs",
        "**/playwright/**/*.spec.js",
        "**/playwright/**/*.spec.jsx",
    ];
    const CARGO: &[&str] = &["**/tests/**/*.rs", "**/src/**/tests.rs", "src/**/*_test.rs"];
    const DOTNET: &[&str] = &["**/Tests/**/*.cs", "**/*.Tests/**/*.cs", "**/*Tests/**/*.cs"];
    const SWIFT: &[&str] = &["**/Tests/**/*.swift"];

    match framework {
        "vitest" => globs_to_strings(VITEST_JEST_TEST_GLOBS),
        "jest" => globs_to_strings(VITEST_JEST_TEST_GLOBS),
        "playwright" => globs_to_strings(PLAYWRIGHT),
        "cargo" => globs_to_strings(CARGO),
        "dotnet" => globs_to_strings(DOTNET),
        "swift" => globs_to_strings(SWIFT),
        "python" => globs_to_strings(&[
            "**/test_*.py",
            "**/*_test.py",
            "**/tests.py",
            "**/tests/**/*.py",
        ]),
        "go" => globs_to_strings(&["**/*_test.go"]),
        "rails" => globs_to_strings(&["**/spec/**/*_spec.rb", "**/test/**/*_test.rb"]),
        "php" => globs_to_strings(&["**/*Test.php", "**/tests/**/*.php"]),
        _ => vec![],
    }
}

fn globs_to_strings(globs: &[&str]) -> Vec<String> {
    globs.iter().map(|&s| s.to_string()).collect()
}
