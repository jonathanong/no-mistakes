const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { readChangedFiles } = require("./pr-diff-scope.cjs");

const INDEPENDENT_PREFIXES = [
  "packages/eslint-plugin-no-mistakes/",
  "test-cases/eslint-snippets/",
  "plugins/no-mistakes-rules/",
];

const RUST_MARKDOWN_PREFIXES = readFileSync(join(__dirname, "rust-markdown-inputs.txt"), "utf8")
  .trim()
  .split(/\r?\n/);
const MARKDOWN = /\.(?:mdx?|markdown)$/i;
const DOC_TOOL_CONFIG = new Set(["cspell.config.yaml", ".lycheeignore"]);

function isNonRustDocumentation(path) {
  return (
    DOC_TOOL_CONFIG.has(path) ||
    (MARKDOWN.test(path) && !RUST_MARKDOWN_PREFIXES.some((prefix) => path.startsWith(prefix)))
  );
}

function triggersRust(path) {
  if (isNonRustDocumentation(path)) return false;
  if (
    /(^|\/)\.cargo\//.test(path) ||
    /\.rs$/.test(path) ||
    /(^|\/)(?:Cargo\.(?:toml|lock)|rust-toolchain(?:\.toml)?)$/.test(path)
  )
    return true;
  return MARKDOWN.test(path) || !INDEPENDENT_PREFIXES.some((prefix) => path.startsWith(prefix));
}

function rustRequired(changes) {
  return changes.length === 0 || changes.some(({ path }) => triggersRust(path));
}

if (require.main === module) {
  const required = process.argv.includes("--all") || rustRequired(readChangedFiles());
  process.stdout.write(`rust=${required}\n`);
}

module.exports = { isNonRustDocumentation, rustRequired, triggersRust };
