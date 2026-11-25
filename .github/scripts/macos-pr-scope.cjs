const { spawnSync } = require("node:child_process");
const { readChangedFiles } = require("./pr-diff-scope.cjs");
const { cliFixture, napiFixture } = require("./smoke-tests.cjs");

const PORTABLE_PREFIXES = [
  "docs/",
  "fixtures/",
  "test-cases/",
  "packages/eslint-plugin-no-mistakes/",
  "plugins/no-mistakes-rules/",
];

const RULES = "crates/no-mistakes/src/codebase/rules/";
const PORTABLE_FILES = new Set([
  "AGENTS.md",
  "CONTRIBUTING.md",
  "README.md",
  ".github/scripts/lychee-gateway.cjs",
  ".github/scripts/lychee-gateway.test.cjs",
  ".github/scripts/lychee-github-gateway.test.cjs",
]);

const PLATFORM_MARKERS = String.raw`(^|[^a-zA-Z0-9_])(target_[a-z_]+|unix|windows|macos|darwin|fs|process|napi|libc|winapi|windows_sys|Path|PathBuf|OsStr|OsString|Command|read_dir|read_to_string|symlink_metadata)([^a-zA-Z0-9_]|$)|std::(os|env|io|path)|cfg[[:space:]]*!?[[:space:]]*\([[:space:]]*(any|all|not)`;

function ruleFile(path) {
  return path.startsWith(RULES) && path.endsWith(".rs");
}

function platformRules(base, head) {
  for (const sha of [base, head]) {
    if (!/^[0-9a-f]{40}$/.test(sha || "")) throw new Error("Expected macOS scope commit SHAs");
  }
  const result = spawnSync("git", ["grep", "-alzE", PLATFORM_MARKERS, base, head, "--", RULES], {
    encoding: "utf8",
  });
  if (result.status !== 0 && result.status !== 1)
    throw new Error(result.stderr || "Could not read the macOS rule scope");
  return new Set(
    result.stdout
      .split("\0")
      .filter(Boolean)
      .map((entry) => entry.slice(41)),
  );
}

function triggersMacos({ path, status }, sensitiveRules) {
  if (status === "T") return true;
  if ([cliFixture, napiFixture].some((fixture) => path.startsWith(`${fixture}/`))) return true;
  if (PORTABLE_FILES.has(path) || path.endsWith(".snap")) return false;
  if (/^packages\/no-mistakes\/[^/]+\.d\.ts$/.test(path)) return false;
  if (PORTABLE_PREFIXES.some((prefix) => path.startsWith(prefix))) return false;
  if (ruleFile(path)) return sensitiveRules.has(path);
  return (
    /^(?:\.github|scripts|packages\/no-mistakes[^/]*)\//.test(path) ||
    !/\.(?:[cm]?[jt]s|[jt]sx)$/.test(path)
  );
}

const changes = process.argv.includes("--all") ? [] : readChangedFiles();
const sensitiveRules = changes.some(({ path }) => ruleFile(path))
  ? platformRules(process.env.DIFF_BASE, process.env.HEAD_SHA)
  : new Set();
process.stdout.write(
  `macos-smoke=${changes.length === 0 || changes.some((change) => triggersMacos(change, sensitiveRules))}\n`,
);
