const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const repoRoot = join(__dirname, "..", "..");

test("every Dependabot update excludes fixture manifests", () => {
  const source = readFileSync(join(repoRoot, ".github", "dependabot.yml"), "utf8");
  const updateHeaders = [...source.matchAll(/^  - package-ecosystem: ([^\n]+)$/gm)];

  assert.ok(updateHeaders.length > 0, "dependabot.yml must define at least one update");

  for (const [index, header] of updateHeaders.entries()) {
    const nextHeader = updateHeaders[index + 1];
    const update = source.slice(header.index, nextHeader?.index);

    // Fixture manifests are saved test inputs; updating them can desynchronize
    // package manifests from their lockfile and expected-output snapshots.
    assert.match(
      update,
      /^    exclude-paths:\n      - fixtures\/\*\*$/m,
      `${header[1]} Dependabot updates must exclude fixtures/**`,
    );
  }
});
