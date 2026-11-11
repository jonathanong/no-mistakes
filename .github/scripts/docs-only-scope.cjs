const DOC_EXTENSIONS = [".md", ".mdx"];

// Markdown under these paths is fixture, package, plugin, or ESLint test input,
// which only the full test jobs read. Other markdown is read by the
// suite_contracts docs tests, which still run for docs-only changes.
const FULL_TEST_PREFIXES = [
  "crates/",
  "packages/",
  "plugins/",
  "fixtures/",
  "test-cases/",
  "tests/",
  "evals/",
  ".ast-grep/",
  "docs/eslint-rules/",
];

// Docs pages that unit or package tests read. tests/js/docs-only-scope.test.js
// fails when a test reads a repository doc that is missing here.
const FULL_TEST_FILES = new Set([
  "docs/cli/diagnostics.md",
  "docs/cli/tests-audit.md",
  "docs/eslint-plugin.md",
  "docs/graph-edges.md",
  "docs/rules/test-no-dependency-pins.md",
]);

function isDocsOnlyPath(path) {
  if (!DOC_EXTENSIONS.some((extension) => path.endsWith(extension))) return false;
  if (FULL_TEST_FILES.has(path)) return false;
  return !FULL_TEST_PREFIXES.some((prefix) => path.startsWith(prefix));
}

function docsOnly(changes) {
  return changes.length > 0 && changes.every(({ path }) => isDocsOnlyPath(path));
}

function readChangedFiles() {
  const fields = require("node:fs").readFileSync(0).toString("utf8").split("\0");
  if (fields.pop() !== "" || fields.length % 2 !== 0) {
    throw new Error("Expected NUL-terminated git diff name-status pairs");
  }
  const changes = [];
  for (let index = 0; index < fields.length; index += 2) {
    const status = fields[index];
    const path = fields[index + 1];
    if (!/^[ADMT]$/.test(status) || !path) {
      throw new Error("Expected added, deleted, modified, or type-changed git diff path");
    }
    changes.push({ status, path });
  }
  return changes;
}

if (require.main === module) {
  if (process.argv.includes("--all")) {
    process.stdout.write("docs-only=false\n");
    process.exit(0);
  }
  process.stdout.write(`docs-only=${docsOnly(readChangedFiles())}\n`);
}

module.exports = { docsOnly, isDocsOnlyPath };
