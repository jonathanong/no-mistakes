const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { mkdtempSync, readdirSync, readFileSync, rmSync, statSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join, posix } = require("node:path");

const root = join(__dirname, "..", "..");
const releaseVersion = JSON.parse(readFileSync(join(root, "package.json"), "utf8")).version;
test("only the expected public npm packages remain", () => {
  const packagesDir = join(root, "packages");
  const manifests = [];

  for (const name of readdirSync(packagesDir)) {
    const manifestPath = join(packagesDir, name, "package.json");
    let manifest;
    try {
      manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    } catch (error) {
      if (error.code === "ENOENT") {
        continue;
      }
      throw error;
    }

    if (manifest.private) {
      continue;
    }
    manifests.push(manifest.name);
    for (const field of ["dependencies", "devDependencies", "peerDependencies"]) {
      if (manifest[field]?.["no-mistakes-core"]) {
        assert.fail(`${manifest.name}:${field} must not depend on no-mistakes-core`);
      }
    }
  }

  assert.deepEqual(manifests.sort(), [
    "eslint-plugin-no-mistakes",
    "no-mistakes",
    "no-mistakes-darwin-arm64",
    "no-mistakes-linux-arm64-gnu",
    "no-mistakes-linux-x64-gnu",
    "no-mistakes-win32-x64-msvc",
  ]);
});

test("native platform package manifests are runtime-bearing and platform-constrained", () => {
  const expected = {
    "no-mistakes-darwin-arm64": { os: ["darwin"], cpu: ["arm64"] },
    "no-mistakes-linux-arm64-gnu": { os: ["linux"], cpu: ["arm64"], libc: ["glibc"] },
    "no-mistakes-linux-x64-gnu": { os: ["linux"], cpu: ["x64"], libc: ["glibc"] },
    "no-mistakes-win32-x64-msvc": { os: ["win32"], cpu: ["x64"] },
  };

  for (const [name, platform] of Object.entries(expected)) {
    const packageDir = join(root, "packages", name);
    const manifest = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
    assert.equal(manifest.name, name);
    assert.equal(manifest.version, releaseVersion);
    assert.deepEqual(manifest.os, platform.os);
    assert.deepEqual(manifest.cpu, platform.cpu);
    assert.deepEqual(manifest.libc, platform.libc);
    assert.equal(manifest.main, "bin/no-mistakes.node");
    assert.equal(manifest.bin, undefined);
    assert.deepEqual(manifest.files, ["bin/", "README.md", "LICENSE"]);
    assert.deepEqual(manifest.publishConfig, { access: "public" });
    assert.equal(manifest.repository.directory, `packages/${name}`);
    assert.equal(manifest.exports, undefined);
    assert.equal(manifest.scripts, undefined);
  }
});

test("the npm package exposes one JavaScript launcher and optional native packages", () => {
  const packageDir = join(root, "packages", "no-mistakes");
  const manifest = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
  assert.deepEqual(manifest.bin, { "no-mistakes": "bin/no-mistakes.js" });
  assert.notEqual(statSync(join(packageDir, manifest.bin["no-mistakes"])).mode & 0o111, 0);

  assert.deepEqual(Object.values(manifest.optionalDependencies), Array(4).fill(releaseVersion));
  assert.equal(statSync(join(packageDir, "bin", "no-mistakes.js")).isFile(), true);
  assert.match(
    readFileSync(join(root, "pnpm-workspace.yaml"), "utf8"),
    /^linkWorkspacePackages: true$/mu,
  );
});

test("packed no-mistakes pins every platform optional dependency to its release version", () => {
  const packageDir = join(root, "packages", "no-mistakes");
  let tarball;
  try {
    execFileSync("pnpm", ["--filter", "no-mistakes", "pack"], { cwd: root, stdio: "pipe" });
    tarball = join(root, `no-mistakes-${releaseVersion}.tgz`);
    const packed = JSON.parse(
      execFileSync("tar", ["-xOf", tarball, "package/package.json"], { encoding: "utf8" }),
    );
    assert.deepEqual(Object.values(packed.optionalDependencies), Array(4).fill(packed.version));
  } finally {
    if (tarball) rmSync(tarball, { force: true });
  }
});

// A `require("./sibling")` in a published entry point that isn't covered by
// `package.json`'s `files` list is absent from the published tarball —
// `require("no-mistakes")` throws MODULE_NOT_FOUND for every consumer, not
// just the feature that needed the missing file. Caught once already
// (`workflow-topology-index.js`); this generalizes the check so the next
// sibling module can't repeat it.
test("npm pack includes every transitive local require() from a published entry point", () => {
  const packageDir = join(root, "packages", "no-mistakes");
  const manifest = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
  const entryPoints = [
    "index.js",
    "planning.js",
    "workflow-topology-index.js",
    ...Object.values(manifest.bin || {}),
  ];
  const requirePattern = /require\("(\.{1,2}\/[\w./-]+)"\)/g;
  const npmCache = mkdtempSync(join(tmpdir(), "no-mistakes-npm-pack-"));
  let packed;
  try {
    const npmArguments = ["pack", "--dry-run", "--ignore-scripts", "--json"];
    const command = process.platform === "win32" ? process.env.ComSpec || "cmd.exe" : "npm";
    const commandArguments =
      process.platform === "win32" ? ["/d", "/s", "/c", "npm", ...npmArguments] : npmArguments;
    packed = JSON.parse(
      execFileSync(command, commandArguments, {
        cwd: packageDir,
        encoding: "utf8",
        env: { ...process.env, NPM_CONFIG_CACHE: npmCache },
      }),
    );
  } finally {
    rmSync(npmCache, { recursive: true, force: true });
  }
  const packedPaths = new Set(packed[0].files.map((file) => file.path));
  const packedLauncher = packed[0].files.find((file) => file.path === manifest.bin["no-mistakes"]);
  assert.ok(packedLauncher, "npm pack must include the public launcher");
  assert.notEqual(packedLauncher.mode & 0o111, 0);

  const isCovered = (relativePath) =>
    manifest.files.some((pattern) => {
      if (pattern.endsWith("/")) return relativePath.startsWith(pattern);
      if (pattern.includes("*")) {
        const escaped = pattern
          .split("*")
          .map((segment) => segment.replace(/[.+?^${}()|[\]\\]/g, "\\$&"))
          .join(".*");
        return new RegExp(`^${escaped}$`).test(relativePath);
      }
      return relativePath === pattern;
    });

  const checked = new Set();
  const resolveLocalRequire = (entry, required) => {
    const requestedPath = posix.normalize(posix.join(posix.dirname(entry), required));
    return posix.extname(requestedPath) ? requestedPath : `${requestedPath}.js`;
  };
  assert.equal(resolveLocalRequire("sub/a.js", "b"), "sub/b.js");
  const checkEntry = (entry) => {
    if (checked.has(entry)) return;
    checked.add(entry);
    const source = readFileSync(join(packageDir, entry), "utf8");
    for (const match of source.matchAll(requirePattern)) {
      const required = match[1];
      // `./bin/no-mistakes.node` is the native addon, covered by `bin/`
      // regardless of extension; every other local require resolves to a
      // sibling `.js` file the same way Node's CJS resolver would.
      const relativePath = resolveLocalRequire(entry, required);
      assert.ok(
        isCovered(relativePath),
        `${entry} requires "./${required}" but ${relativePath} is not covered by package.json's files list`,
      );
      assert.ok(
        packedPaths.has(relativePath),
        `${entry} requires "./${required}" but npm pack does not include ${relativePath}`,
      );
      if (!relativePath.startsWith("bin/")) checkEntry(relativePath);
    }
  };

  entryPoints.forEach(checkEntry);
});
