const { spawnSync } = require("node:child_process");

const TRACKED_PACKAGES = ["vitest", "@vitest/browser", "@vitest/browser-playwright", "playwright"];

const ALWAYS_PATHS = new Set([
  ".github/workflows/ci.yml",
  ".github/scripts/timeout-policy-browser-scope.cjs",
  "tests/js/timeout-policy-browser-scope.test.js",
  "vitest.config.js",
  "packages/no-mistakes/package.json",
]);

function isAlwaysPath(path) {
  return (
    ALWAYS_PATHS.has(path) ||
    path.startsWith("packages/no-mistakes/test-timeout-policy/") ||
    path.startsWith("fixtures/test-timeout-policy/")
  );
}

function dependencyVersions(source) {
  const pkg = JSON.parse(source);
  const bags = [
    pkg.dependencies,
    pkg.devDependencies,
    pkg.optionalDependencies,
    pkg.peerDependencies,
  ];
  return TRACKED_PACKAGES.map((name) => {
    let version = "";
    for (const bag of bags) {
      if (bag && bag[name]) version = String(bag[name]);
    }
    return `${name}@${version}`;
  }).join("\n");
}

function lockfileFingerprint(source) {
  return source
    .split("\n")
    .filter((line) =>
      TRACKED_PACKAGES.some((name) => line.includes(`${name}@`) || line.includes(`${name}:`)),
    )
    .join("\n");
}

function snapshotsDiffer(base, head, extract) {
  if (base == null || head == null) return true;
  return extract(base) !== extract(head);
}

function timeoutPolicyBrowserChanged(changes, compared = {}) {
  for (const { path } of changes) {
    if (isAlwaysPath(path)) return true;
  }
  const pkgTouched = changes.some(({ path }) => path === "package.json");
  const lockTouched = changes.some(({ path }) => path === "pnpm-lock.yaml");
  if (!pkgTouched && !lockTouched) return false;
  const pkg = compared.packageJson || {};
  const lock = compared.lockfile || {};
  return (
    (pkgTouched && snapshotsDiffer(pkg.base, pkg.head, dependencyVersions)) ||
    (lockTouched && snapshotsDiffer(lock.base, lock.head, lockfileFingerprint))
  );
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

function gitShow(sha, path) {
  const result = spawnSync("git", ["show", `${sha}:${path}`], { encoding: "utf8" });
  return result.status === 0 ? result.stdout : null;
}

function comparedFiles(changes) {
  const base = process.env.BASE_SHA;
  const head = process.env.HEAD_SHA;
  const compared = {};
  if (!base || !head) return compared;
  if (changes.some(({ path }) => path === "package.json")) {
    compared.packageJson = {
      base: gitShow(base, "package.json"),
      head: gitShow(head, "package.json"),
    };
  }
  if (changes.some(({ path }) => path === "pnpm-lock.yaml")) {
    compared.lockfile = {
      base: gitShow(base, "pnpm-lock.yaml"),
      head: gitShow(head, "pnpm-lock.yaml"),
    };
  }
  return compared;
}

if (require.main === module) {
  if (process.argv.includes("--all")) {
    process.stdout.write("changed=true\n");
    process.exit(0);
  }
  const changes = readChangedFiles();
  process.stdout.write(`changed=${timeoutPolicyBrowserChanged(changes, comparedFiles(changes))}\n`);
}

module.exports = { timeoutPolicyBrowserChanged };
