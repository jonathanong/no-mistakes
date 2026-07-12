const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { mkdir, mkdtemp, readFile, rm, stat, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { tmpdir } = require("node:os");
const { pathToFileURL } = require("node:url");
const core = require("./install/index");

const { platformTarget, unsupportedPlatformMessage } = core;
const repository = "jonathanong/no-mistakes";

function assetName(version, target) {
  return core.assetName({ binName, version, target });
}

function install(options) {
  return core.install(binName, repository, {
    destinationName: binName,
    envVar: "NO_MISTAKES_RELEASE_BASE_URL",
    ...options,
  });
}

function assetBaseUrl(root) {
  return pathToFileURL(join(root, "assets")).toString();
}

function executableName() {
  return "no-mistakes";
}

const binName = "no-mistakes";
const version = "9.8.7";

test("rejects unsupported install targets", async () => {
  await assert.rejects(
    () => core.install("simple", "owner/repo", { target: "x86_64-unknown-linux-gnu" }),
    /version is required/,
  );
  await assert.rejects(
    () => core.install("simple", "owner/repo", { target: "x86_64-unknown-linux-gnu", version }),
    /vendorDir is required/,
  );
  await assert.rejects(() => install({ target: null, version }), /Unsupported platform/);
  assert.match(
    unsupportedPlatformMessage(binName, "freebsd", "x64"),
    /Unsupported platform freebsd\/x64/,
  );
  assert.match(
    unsupportedPlatformMessage(binName, "linux", "x64", { getReport: () => ({ header: {} }) }),
    /glibc 2\.35/,
  );
  assert.match(
    unsupportedPlatformMessage(binName, "linux", "arm64", { getReport: () => ({ header: {} }) }),
    /glibc 2\.35/,
  );
});

test("skips existing binaries when requested and the installed version matches", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-existing-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";
  const existing = join(vendorDir, executableName(target));

  try {
    await mkdir(vendorDir, { recursive: true });
    await writeFile(existing, "already here");
    await writeFile(core.versionMarkerPath(existing), version);
    assert.equal(await install({ checkExisting: true, target, vendorDir, version }), existing);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

// Regression test: checkExisting must not treat "a file happens to be at the
// destination" as "the requested version is already installed." Without a
// version marker (e.g. a binary left behind by an older no-mistakes release
// whose postinstall predates version marking, or one belonging to a
// different version that a package manager didn't fully replace on upgrade),
// checkExisting previously skipped the download unconditionally — silently
// stranding users on a stale binary indefinitely. It must fall through to a
// real (re)download whenever the marker is missing or names a different
// version, then leave a marker for the version it actually installed.
test("redownloads when checkExisting finds no version marker or a stale one", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-stale-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";
  const destination = join(vendorDir, executableName(target));
  const asset = assetName(version, target);
  const content = Buffer.from("#!/bin/sh\nexit 0\n");
  const hash = createHash("sha256").update(content).digest("hex");

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), content);
  await writeFile(join(root, "assets", `${asset}.sha256`), `${hash}  ${asset}\n`);

  try {
    // Case 1: destination exists but has no version marker at all (as if left
    // by a pre-version-marking install).
    await mkdir(vendorDir, { recursive: true });
    await writeFile(destination, "old binary, no marker");
    await install({ baseUrl: assetBaseUrl(root), checkExisting: true, target, vendorDir, version });
    assert.equal(await readFile(destination, "utf8"), content.toString("utf8"));
    assert.equal(await readFile(core.versionMarkerPath(destination), "utf8"), version);

    // Case 2: destination exists with a marker naming a different (older) version.
    await writeFile(destination, "old binary, stale marker");
    await writeFile(core.versionMarkerPath(destination), "1.0.0");
    await install({ baseUrl: assetBaseUrl(root), checkExisting: true, target, vendorDir, version });
    assert.equal(await readFile(destination, "utf8"), content.toString("utf8"));
    assert.equal(await readFile(core.versionMarkerPath(destination), "utf8"), version);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("installedVersion propagates unexpected read errors instead of treating them as missing", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-marker-error-"));
  const vendorDir = join(root, "vendor");
  const destination = join(vendorDir, executableName("x86_64-unknown-linux-gnu"));

  try {
    await mkdir(vendorDir, { recursive: true });
    // A directory at the marker path triggers EISDIR, not ENOENT.
    await mkdir(core.versionMarkerPath(destination));
    assert.throws(() => core.installedVersion(destination), /EISDIR/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("cleans up temporary file and throws error on checksum mismatch", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-error-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";
  const asset = assetName(version, target);
  const content = Buffer.from("#!/bin/sh\nexit 0\n");
  const badHash = createHash("sha256").update(Buffer.from("wrong content")).digest("hex");

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), content);
  await writeFile(join(root, "assets", `${asset}.sha256`), `${badHash}  ${asset}\n`);

  try {
    await assert.rejects(
      install({
        baseUrl: assetBaseUrl(root),
        checkExisting: true,
        target,
        vendorDir,
        version,
      }),
      /Failed to install no-mistakes: Checksum mismatch/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("installs only the requested platform binary and verifies checksum", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-test-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";
  const asset = assetName(version, target);
  const content = Buffer.from("#!/bin/sh\nexit 0\n");
  const hash = createHash("sha256").update(content).digest("hex");

  await mkdir(join(root, "assets"));
  await mkdir(vendorDir, { recursive: true });
  await writeFile(join(vendorDir, executableName(target)), "Native binary placeholder.\n");
  await writeFile(join(root, "assets", asset), content);
  await writeFile(join(root, "assets", `${asset}.sha256`), `${hash}  ${asset}\n`);
  await writeFile(join(root, "assets", "no-mistakes-v9.8.7-aarch64-apple-darwin"), "nope");

  try {
    const installed = await install({
      baseUrl: assetBaseUrl(root),
      checkExisting: true,
      target,
      vendorDir,
      version,
    });
    assert.equal(installed, join(vendorDir, executableName(target)));
    assert.equal(await readFile(installed, "utf8"), content.toString("utf8"));
    if (process.platform !== "win32") {
      assert.equal((await stat(installed)).mode & 0o111, 0o111);
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("wraps checksum download failures", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-missing-checksum-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";
  const asset = assetName(version, target);

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), "binary");

  try {
    await assert.rejects(
      () => install({ baseUrl: assetBaseUrl(root), target, vendorDir, version }),
      /Failed to fetch checksum/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("installs with default target and release base environment", async () => {
  const previous = process.env.NO_MISTAKES_RELEASE_BASE_URL;
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-env-install-"));
  const vendorDir = join(root, "vendor");
  const target = platformTarget();
  if (!target) {
    await rm(root, { recursive: true, force: true });
    return;
  }
  const asset = assetName(version, target);
  const content = Buffer.from("#!/bin/sh\nexit 0\n");
  const hash = createHash("sha256").update(content).digest("hex");

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), content);
  await writeFile(join(root, "assets", `${asset}.sha256`), `${hash}  ${asset}\n`);

  try {
    process.env.NO_MISTAKES_RELEASE_BASE_URL = assetBaseUrl(root);
    const installed = await install({ vendorDir, version });
    assert.equal(installed, join(vendorDir, executableName(target)));
  } finally {
    if (previous === undefined) {
      delete process.env.NO_MISTAKES_RELEASE_BASE_URL;
    } else {
      process.env.NO_MISTAKES_RELEASE_BASE_URL = previous;
    }
    await rm(root, { recursive: true, force: true });
  }
});
