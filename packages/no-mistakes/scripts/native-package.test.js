const assert = require("node:assert/strict");
const { chmodSync, mkdtempSync, mkdirSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const {
  localPackagePath,
  nativePackageName,
  resolveNativePackage,
  supportedGlibc,
  unsupportedGlibcMessage,
  unsupportedPlatformMessage,
  usableCli,
} = require("./native-package");

test("accepts only regular executable Unix files and regular Windows files", () => {
  const directory = mkdtempSync(join(tmpdir(), "no-mistakes-cli-mode-"));
  try {
    const cli = join(directory, "no-mistakes");
    const childDirectory = join(directory, "directory");
    writeFileSync(cli, "binary");
    mkdirSync(childDirectory);
    chmodSync(cli, 0o644);
    if (process.platform !== "win32") assert.equal(usableCli(cli, "darwin"), false);
    assert.equal(usableCli(cli, "win32"), true);
    assert.equal(usableCli(childDirectory, "win32"), false);
    chmodSync(cli, 0o755);
    if (process.platform !== "win32") assert.equal(usableCli(cli, "darwin"), true);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("finds the sibling platform workspace only for repository staging", () => {
  assert.match(localPackagePath("no-mistakes-darwin-arm64"), /no-mistakes-darwin-arm64$/);
  assert.equal(localPackagePath("not-a-no-mistakes-package"), undefined);
});

test("maps supported Node platforms to their native optional package", () => {
  assert.equal(nativePackageName("darwin", "arm64"), "no-mistakes-darwin-arm64");
  assert.equal(nativePackageName("darwin", "x64"), undefined);
  assert.equal(nativePackageName("linux", "arm64"), "no-mistakes-linux-arm64-gnu");
  assert.equal(nativePackageName("linux", "x64"), "no-mistakes-linux-x64-gnu");
  assert.equal(nativePackageName("win32", "x64"), "no-mistakes-win32-x64-msvc");
  assert.equal(nativePackageName("win32", "arm64"), undefined);
});

test("accepts Linux native packages only on glibc 2.35 or newer", () => {
  const report = (version) => ({ getReport: () => ({ header: { glibcVersionRuntime: version } }) });
  assert.equal(supportedGlibc(report("2.35")), true);
  assert.equal(supportedGlibc(report("2.36")), true);
  assert.equal(supportedGlibc(report("3.0")), true);
  assert.equal(supportedGlibc(report("2.34")), false);
  assert.equal(supportedGlibc(report("invalid")), false);
  assert.equal(
    supportedGlibc({ getReport: () => ({ header: { glibcVersionCompiler: "2.35" } }) }),
    true,
  );
  assert.equal(supportedGlibc({}), false);
  assert.equal(
    supportedGlibc({
      getReport: () => {
        throw new Error("unavailable");
      },
    }),
    false,
  );
});

test("rejects unsupported Linux glibc before resolving native packages", () => {
  assert.throws(
    () =>
      resolveNativePackage({
        platform: "linux",
        arch: "x64",
        report: { getReport: () => ({ header: { glibcVersionRuntime: "2.34" } }) },
        resolve: () => assert.fail("unsupported glibc must not resolve a package"),
      }),
    /glibc 2\.35 or newer\. Detected glibc 2\.34\. Install with `cargo install no-mistakes`/,
  );
  assert.match(unsupportedGlibcMessage({}), /Could not detect a supported glibc runtime/);
});

test("resolves the CLI and addon from one selected optional package", () => {
  const resolved = resolveNativePackage({
    platform: "darwin",
    arch: "arm64",
    resolve: (request) => `/virtual/${request}`,
    isUsableCli: () => true,
  });
  assert.deepEqual(resolved, {
    name: "no-mistakes-darwin-arm64",
    cliPath: "/virtual/no-mistakes-darwin-arm64/bin/no-mistakes",
    addonPath: "/virtual/no-mistakes-darwin-arm64",
  });
});

test("resolves the Windows executable from the same optional package", () => {
  const resolved = resolveNativePackage({
    platform: "win32",
    arch: "x64",
    resolve: (request) => `/virtual/${request}`,
    isUsableCli: () => true,
  });
  assert.equal(resolved.cliPath, "/virtual/no-mistakes-win32-x64-msvc/bin/no-mistakes.exe");
});

test("reports missing optional packages and unsupported platforms clearly", () => {
  assert.throws(
    () =>
      resolveNativePackage({
        platform: "darwin",
        arch: "arm64",
        resolve: () => {
          throw new Error("absent");
        },
        resolveLocalPackage: () => undefined,
        isUsableCli: () => true,
      }),
    /no-mistakes-darwin-arm64.*npm install/i,
  );
  assert.match(unsupportedPlatformMessage("win32", "arm64"), /Unsupported platform win32\/arm64/);
  assert.throws(
    () => resolveNativePackage({ platform: "darwin", arch: "x64" }),
    /Unsupported platform darwin\/x64\. Install with `cargo install no-mistakes` instead\./,
  );
  assert.throws(
    () =>
      resolveNativePackage({
        platform: "linux",
        arch: "riscv64",
        report: {
          getReport: () => assert.fail("unsupported architectures must not inspect glibc"),
        },
      }),
    /Unsupported platform linux\/riscv64/,
  );
});

test("directs an installed package missing its CLI to repair optional dependencies", () => {
  assert.throws(
    () =>
      resolveNativePackage({
        platform: "darwin",
        arch: "arm64",
        resolve: (request) =>
          request.endsWith("package.json")
            ? "/virtual/package.json"
            : (() => {
                throw new Error("absent");
              })(),
        resolveLocalPackage: () => undefined,
        isUsableCli: () => true,
      }),
    /unavailable.*npm install/i,
  );
});

test("uses the local staged native package for repository development", () => {
  const resolved = resolveNativePackage({
    platform: "darwin",
    arch: "arm64",
    resolve: () => {
      throw new Error("not installed");
    },
    resolveLocalPackage: () => "/repo/packages/no-mistakes-darwin-arm64",
    localArtifactExists: () => true,
    isUsableCli: () => true,
  });
  assert.deepEqual(resolved, {
    name: "no-mistakes-darwin-arm64",
    cliPath: "/repo/packages/no-mistakes-darwin-arm64/bin/no-mistakes",
    addonPath: "/repo/packages/no-mistakes-darwin-arm64/bin/no-mistakes.node",
  });
});

test("rejects an installed Unix CLI without execute permission before spawn", () => {
  assert.throws(
    () =>
      resolveNativePackage({
        platform: "darwin",
        arch: "arm64",
        resolve: (request) => `/virtual/${request}`,
        resolveLocalPackage: () => undefined,
        isUsableCli: () => false,
      }),
    /not executable.*npm install/i,
  );
});

test("rejects a local staged Unix CLI without execute permission before spawn", () => {
  assert.throws(
    () =>
      resolveNativePackage({
        platform: "darwin",
        arch: "arm64",
        resolve: () => {
          throw new Error("not installed");
        },
        resolveLocalPackage: () => "/repo/packages/no-mistakes-darwin-arm64",
        localArtifactExists: () => true,
        isUsableCli: () => false,
      }),
    /unusable CLI artifact.*pnpm run build:native/i,
  );
});

test("directs a local package with missing artifacts to explicit staging", () => {
  assert.throws(
    () =>
      resolveNativePackage({
        platform: "darwin",
        arch: "arm64",
        resolve: () => {
          throw new Error("not installed");
        },
        resolveLocalPackage: () => "/repo/packages/no-mistakes-darwin-arm64",
        localArtifactExists: () => false,
      }),
    /no staged artifacts.*pnpm run build:native/i,
  );
});

test("Windows verifies CLI presence without Unix execute-bit semantics", () => {
  const resolved = resolveNativePackage({
    platform: "win32",
    arch: "x64",
    resolve: (request) => `/virtual/${request}`,
    isUsableCli: (path, platform) => {
      assert.equal(platform, "win32");
      assert.match(path, /\.exe$/);
      return true;
    },
  });
  assert.match(resolved.cliPath, /\.exe$/);
});
