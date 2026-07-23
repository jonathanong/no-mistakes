const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const repoRoot = join(__dirname, "..", "..");

function sequenceValues(source, key) {
  const lines = source.split("\n");
  const keyIndex = lines.findIndex((line) => line.trim() === `${key}:`);
  if (keyIndex === -1) {
    return [];
  }

  const keyIndent = lines[keyIndex].search(/\S/);
  const values = [];
  for (const line of lines.slice(keyIndex + 1)) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }
    if (line.search(/\S/) <= keyIndent) {
      break;
    }

    const item = trimmed.match(/^-\s+(.+)$/);
    if (item) {
      values.push(item[1].replace(/^(["'])(.*)\1$/, "$2"));
    }
  }
  return values;
}

test("every Dependabot update excludes fixture manifests", () => {
  const source = readFileSync(join(repoRoot, ".github", "dependabot.yml"), "utf8");
  const updateHeaders = [...source.matchAll(/^  - package-ecosystem: ([^\n]+)$/gm)];

  assert.ok(updateHeaders.length > 0, "dependabot.yml must define at least one update");

  for (const [index, header] of updateHeaders.entries()) {
    const nextHeader = updateHeaders[index + 1];
    const update = source.slice(header.index, nextHeader?.index);

    // Fixture manifests are saved test inputs; updating them can desynchronize
    // package manifests from their lockfile and expected-output snapshots.
    assert.ok(
      sequenceValues(update, "exclude-paths").includes("fixtures/**"),
      `${header[1]} Dependabot updates must exclude fixtures/**`,
    );
  }
});
