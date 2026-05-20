const assert = require("node:assert/strict");
const { readdirSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const root = join(__dirname, "..", "..");

test("publishable npm packages do not depend on no-mistakes-core", () => {
  const packagesDir = join(root, "packages");
  const offenders = [];

  for (const name of readdirSync(packagesDir)) {
    const manifestPath = join(packagesDir, name, "package.json");
    let manifest;
    try {
      manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    } catch {
      continue;
    }

    if (manifest.private) {
      continue;
    }
    for (const field of ["dependencies", "devDependencies", "peerDependencies"]) {
      if (manifest[field]?.["no-mistakes-core"]) {
        offenders.push(`${manifest.name}:${field}`);
      }
    }
  }

  assert.deepEqual(offenders, []);
});
