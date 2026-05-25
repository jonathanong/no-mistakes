const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { mkdir, mkdtemp, rm, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { tmpdir } = require("node:os");
const { pathToFileURL } = require("node:url");
const core = require("./install/index");

const binName = "no-mistakes";
const version = "9.8.7";
const repository = "jonathanong/no-mistakes";

function assetName(versionValue, target) {
  return core.assetName(binName, versionValue, target);
}

function assetBaseUrl(root) {
  return pathToFileURL(join(root, "assets")).toString();
}

function install(options) {
  return core.install(binName, repository, {
    destinationName: binName,
    envVar: "NO_MISTAKES_RELEASE_BASE_URL",
    ...options,
  });
}

test("installs Windows assets without chmod", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-windows-install-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-pc-windows-msvc";
  const asset = assetName(version, target);
  const content = Buffer.from("windows");
  const hash = createHash("sha256").update(content).digest("hex");

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), content);
  await writeFile(join(root, "assets", `${asset}.sha256`), `${hash}  ${asset}\n`);

  try {
    const installed = await install({
      baseUrl: assetBaseUrl(root),
      target,
      vendorDir,
      version,
    });
    assert.equal(installed, join(vendorDir, "no-mistakes"));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("rejects checksum mismatches and cleans temporary files", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-bad-checksum-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";
  const asset = assetName(version, target);
  const content = Buffer.from("#!/bin/sh\nexit 0\n");

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), content);
  await writeFile(join(root, "assets", `${asset}.sha256`), `${"b".repeat(64)}  ${asset}\n`);

  try {
    await assert.rejects(
      () => install({ baseUrl: assetBaseUrl(root), target, vendorDir, version }),
      /Checksum mismatch/,
    );
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
