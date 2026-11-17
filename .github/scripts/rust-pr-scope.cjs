const { readChangedFiles } = require("./macos-pr-scope.cjs");

const INDEPENDENT_PREFIXES = [
  "packages/eslint-plugin-no-mistakes/",
  "test-cases/eslint-snippets/",
  "plugins/no-mistakes-rules/",
  "docs/eslint-rules/",
];

function triggersRust(path) {
  if (
    /(^|\/)\.cargo\//.test(path) ||
    /\.rs$/.test(path) ||
    /(^|\/)(?:Cargo\.(?:toml|lock)|rust-toolchain(?:\.toml)?)$/.test(path)
  )
    return true;
  return !INDEPENDENT_PREFIXES.some((prefix) => path.startsWith(prefix));
}

function rustRequired(changes) {
  return changes.length === 0 || changes.some(({ path }) => triggersRust(path));
}

if (require.main === module) {
  const required = process.argv.includes("--all") || rustRequired(readChangedFiles());
  process.stdout.write(`rust=${required}\n`);
}

module.exports = { rustRequired, triggersRust };
