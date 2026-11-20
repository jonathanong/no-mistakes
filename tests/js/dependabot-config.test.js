const assert = require("node:assert/strict");
const { readFileSync, readdirSync } = require("node:fs");
const { join, relative } = require("node:path");
const test = globalThis.test || require("node:test").test;

const repoRoot = join(__dirname, "..", "..");
const fixtureExclusions = ["fixtures/**", "test-cases/**"];
const fixtureDirectories = ["/fixtures", "/fixtures/**", "/test-cases", "/test-cases/**"];
// Fixture directories whose behavior depends on the real package version.
// Each one must be omitted from the blanket fixture ignore and given its own
// Dependabot entry that does not ignore every dependency.
const versionSensitiveDirectories = [];

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

function updateBlocks(source) {
  const headers = [...source.matchAll(/^  - package-ecosystem: ([^\n]+)$/gm)];
  return headers.map((header, index) => {
    const next = headers[index + 1];
    return {
      ecosystem: header[1],
      body: source.slice(header.index, next?.index),
    };
  });
}

function ignoreItems(update) {
  const lines = update.split("\n");
  const ignoreIndex = lines.findIndex((line) => line.trim() === "ignore:");
  if (ignoreIndex === -1) {
    return [];
  }

  const ignoreIndent = lines[ignoreIndex].search(/\S/);
  const items = [];
  let current = null;
  for (const line of lines.slice(ignoreIndex + 1)) {
    if (!line.trim()) {
      if (current) {
        current.push(line);
      }
      continue;
    }
    const indent = line.search(/\S/);
    if (indent <= ignoreIndent) {
      break;
    }
    if (indent === ignoreIndent + 2 && line.trim().startsWith("- ")) {
      current = [line];
      items.push(current);
      continue;
    }
    if (current) {
      current.push(line);
    }
  }
  return items.map((item) => item.join("\n"));
}

function ignoresEveryDependency(update) {
  return ignoreItems(update).some((item) => {
    const name = item.match(/dependency-name:\s*(["'])([^"']+)\1/);
    if (!name || name[2] !== "*") {
      return false;
    }
    // update-types does not apply to security updates. An explicit versions
    // range does, and ">= 0" is every release.
    return !item.includes("update-types:") && sequenceValues(item, "versions").includes(">= 0");
  });
}

function isFixtureSilence(update) {
  const directories = sequenceValues(update, "directories");
  return (
    fixtureDirectories.every((pattern) => directories.includes(pattern)) &&
    ignoresEveryDependency(update)
  );
}

function patternCovers(pattern, directory) {
  const target = `/${directory.replaceAll("\\", "/")}`;
  if (pattern.endsWith("/**")) {
    const root = pattern.slice(0, -3);
    return target === root || target.startsWith(`${root}/`);
  }
  return target === pattern;
}

function isVersionedSpecifier(specifier) {
  if (typeof specifier === "number") {
    return true;
  }
  return (
    typeof specifier === "string" &&
    /[0-9]/.test(specifier) &&
    !/^(workspace:|file:|link:|catalog:|portal:)/.test(specifier)
  );
}

function walkPackageJson(directory, found) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name === ".git") {
      continue;
    }
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      walkPackageJson(path, found);
    } else if (entry.name === "package.json") {
      found.push(path);
    }
  }
}

test("every Dependabot update excludes test fixture manifests", () => {
  const source = readFileSync(join(repoRoot, ".github", "dependabot.yml"), "utf8");
  const updates = updateBlocks(source);

  assert.ok(updates.length > 0, "dependabot.yml must define at least one update");

  for (const update of updates) {
    if (isFixtureSilence(update.body)) {
      continue;
    }

    // Fixture manifests are saved test inputs; updating them can desynchronize
    // package manifests from their lockfile and expected-output snapshots.
    const excludePaths = sequenceValues(update.body, "exclude-paths");
    for (const fixtureExclusion of fixtureExclusions) {
      assert.ok(
        excludePaths.includes(fixtureExclusion),
        `${update.ecosystem} Dependabot updates must exclude ${fixtureExclusion}`,
      );
    }
  }
});

test("fixture manifests are ignored for security updates unless the version is under test", () => {
  const source = readFileSync(join(repoRoot, ".github", "dependabot.yml"), "utf8");
  const updates = updateBlocks(source);
  const silenced = updates.filter((update) => isFixtureSilence(update.body));

  for (const ecosystem of ["npm", "cargo"]) {
    const match = silenced.find((update) => update.ecosystem === ecosystem);
    assert.ok(match, `${ecosystem} fixtures need an ignore for every dependency`);
    assert.match(match.body, /open-pull-requests-limit:\s*0/);
  }

  const silenceDirectories = silenced.flatMap((update) =>
    sequenceValues(update.body, "directories"),
  );
  const versionedDirectories = [];
  for (const tree of ["fixtures", "test-cases"]) {
    const manifests = [];
    walkPackageJson(join(repoRoot, tree), manifests);
    for (const manifest of manifests) {
      let json;
      try {
        json = JSON.parse(readFileSync(manifest, "utf8"));
      } catch {
        continue;
      }
      const specs = [
        json.dependencies,
        json.devDependencies,
        json.optionalDependencies,
        json.peerDependencies,
      ].flatMap((field) => (field && typeof field === "object" ? Object.values(field) : []));
      if (specs.some(isVersionedSpecifier)) {
        versionedDirectories.push(relative(repoRoot, join(manifest, "..")));
      }
    }
  }

  for (const directory of versionedDirectories) {
    if (versionSensitiveDirectories.includes(directory)) {
      assert.ok(
        !silenced.some(
          (update) =>
            update.ecosystem === "npm" &&
            sequenceValues(update.body, "directories").some((pattern) =>
              patternCovers(pattern, directory),
            ) &&
            !sequenceValues(update.body, "exclude-paths").some((pattern) =>
              patternCovers(pattern.startsWith("/") ? pattern : `/${pattern}`, directory),
            ),
        ),
        `${directory} must be omitted from the blanket fixture ignore`,
      );
      const dedicated = updates.find(
        (update) =>
          update.ecosystem === "npm" &&
          !isFixtureSilence(update.body) &&
          (sequenceValues(update.body, "directories").some((pattern) =>
            patternCovers(pattern, directory),
          ) ||
            sequenceValues(update.body, "directory").includes(`/${directory}`)) &&
          !ignoresEveryDependency(update.body),
      );
      assert.ok(dedicated, `${directory} keeps Dependabot updates`);
      continue;
    }
    assert.ok(
      silenceDirectories.some((pattern) => patternCovers(pattern, directory)),
      `${directory} is a decoy and must be ignored`,
    );
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
