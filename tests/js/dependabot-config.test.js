const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const repoRoot = join(__dirname, "..", "..");
const fixtureExclusions = ["fixtures/**", "test-cases/**"];

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

test("every Dependabot update excludes test fixture manifests", () => {
  const source = readFileSync(join(repoRoot, ".github", "dependabot.yml"), "utf8");
  const updateHeaders = [...source.matchAll(/^  - package-ecosystem: ([^\n]+)$/gm)];

  assert.ok(updateHeaders.length > 0, "dependabot.yml must define at least one update");

  for (const [index, header] of updateHeaders.entries()) {
    const nextHeader = updateHeaders[index + 1];
    const update = source.slice(header.index, nextHeader?.index);

    // Fixture manifests are saved test inputs; updating them can desynchronize
    // package manifests from their lockfile and expected-output snapshots.
    const excludePaths = sequenceValues(update, "exclude-paths");
    for (const fixtureExclusion of fixtureExclusions) {
      assert.ok(
        excludePaths.includes(fixtureExclusion),
        `${header[1]} Dependabot updates must exclude ${fixtureExclusion}`,
      );
    }
  }
});

test("OXC dependencies are aligned and updated as one dedicated group", () => {
  const source = readFileSync(join(repoRoot, ".github", "dependabot.yml"), "utf8");
  const cargo = source
    .split("  - package-ecosystem: cargo\n")[1]
    ?.split("  - package-ecosystem:")[0];
  assert.ok(cargo, "Cargo updates must be configured");
  const oxc = cargo.split("      oxc-dependencies:\n")[1]?.split("      rust-dependencies:")[0];
  assert.ok(oxc, "OXC crates need a dedicated group");
  assert.deepEqual(sequenceValues(oxc, "patterns"), ["oxc_*"]);
  const otherRust = cargo.split("      rust-dependencies:\n")[1];
  assert.ok(otherRust, "Other Rust dependencies keep their existing group");
  assert.ok(sequenceValues(otherRust, "exclude-patterns").includes("oxc_*"));

  const manifest = readFileSync(join(repoRoot, "crates", "no-mistakes", "Cargo.toml"), "utf8");
  const dependencies = [...manifest.matchAll(/^(oxc_\w+)\s*=\s*"([^"]+)"/gm)];
  assert.ok(dependencies.some(([, name]) => name === "oxc_semantic"));
  const versions = new Set(dependencies.map(([, , version]) => version));
  assert.equal(versions.size, 1, "All shared OXC AST/string types must use the same release");
  assert.ok(![...versions][0].startsWith("="), "The temporary migration pin must be removed");
});
