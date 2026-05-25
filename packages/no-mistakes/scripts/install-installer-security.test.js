const assert = require("node:assert/strict");
const { access, mkdir, mkdtemp, readdir, rm, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { tmpdir } = require("node:os");
const { pathToFileURL } = require("node:url");
const core = require("./install/index");

const repository = "jonathanong/no-mistakes";

function assetName(version, target) {
  return core.assetName(binName, version, target);
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

function assetBaseUrlObject(root) {
  return pathToFileURL(join(root, "assets"));
}

const binName = "no-mistakes";
const version = "9.8.7";

test("rejects untrusted base URLs for arbitrary file download mitigation", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-untrusted-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";

  await mkdir(vendorDir, { recursive: true });

  try {
    await assert.rejects(
      () => install({ baseUrl: "https://evil.com/releases", target, vendorDir, version }),
      /Untrusted base URL/,
    );
    await assert.rejects(
      () =>
        install({
          baseUrl: "http://github.com/jonathanong/no-mistakes/releases",
          target,
          vendorDir,
          version,
        }),
      /Untrusted base URL/,
    );
    await assert.rejects(
      () => install({ baseUrl: "file:/tmp/assets", target, vendorDir, version }),
      /Untrusted base URL/,
    );
    await assert.rejects(
      () => install({ baseUrl: "ftp://127.0.0.1/releases", target, vendorDir, version }),
      /Untrusted base URL/,
    );
    await assert.rejects(
      () => install({ baseUrl: "file://remote-share/releases", target, vendorDir, version }),
      /Untrusted base URL/,
    );
    await assert.rejects(
      () =>
        install({ baseUrl: "https://github.com/evil/repo/releases", target, vendorDir, version }),
      /Untrusted GitHub repository/,
    );
    await assert.rejects(
      () => install({ baseUrl: "not-a-url", target, vendorDir, version }),
      /Invalid base URL/,
    );
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
  const destination = join(vendorDir, binName);
  const temp = `${destination}.tmp-${process.pid}`;

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), content);
  await writeFile(join(root, "assets", `${asset}.sha256`), `${"b".repeat(64)}  ${asset}\n`);

  try {
    await assert.rejects(
      () => install({ baseUrl: assetBaseUrl(root), target, vendorDir, version }),
      /Checksum mismatch/,
    );
    await assert.rejects(() => access(temp), { code: "ENOENT" });
    const vendorContents = await readdir(vendorDir);
    assert.equal(
      vendorContents.some((entry) => entry.endsWith(`.tmp-${process.pid}`)),
      false,
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

test("accepts URL object base URLs", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-url-object-"));
  const vendorDir = join(root, "vendor");
  const target = "x86_64-unknown-linux-gnu";
  const asset = assetName(version, target);

  await mkdir(join(root, "assets"));
  await writeFile(join(root, "assets", asset), "binary");

  try {
    await assert.rejects(
      () => install({ baseUrl: assetBaseUrlObject(root), target, vendorDir, version }),
      /Failed to fetch checksum/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
