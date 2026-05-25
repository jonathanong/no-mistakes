const assert = require("node:assert/strict");
const { mkdir, mkdtemp, rm, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { pathToFileURL } = require("node:url");
const { tmpdir } = require("node:os");
const core = require("./install/index");

const repository = "jonathanong/no-mistakes";
const binName = "no-mistakes";
const version = "9.8.7";

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

test("skips binary download when skip flag is set and binary exists", async () => {
  const previousSkip = process.env.SKIP_BINARY_DOWNLOAD;
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-skip-download-"));
  const vendorDir = join(root, "vendor");
  const destination = join(vendorDir, binName);

  await mkdir(vendorDir, { recursive: true });
  await writeFile(destination, "#!/bin/sh\nexit 0\n");

  process.env.SKIP_BINARY_DOWNLOAD = "1";

  try {
    assert.equal(await install({ vendorDir, version }), destination);
  } finally {
    if (previousSkip === undefined) {
      delete process.env.SKIP_BINARY_DOWNLOAD;
    } else {
      process.env.SKIP_BINARY_DOWNLOAD = previousSkip;
    }
    await rm(root, { recursive: true, force: true });
  }
});

test("requires placeholder status for skip env when destination is missing", async () => {
  const previousSkip = process.env.SKIP_BINARY_DOWNLOAD;
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-skip-missing-"));
  const vendorDir = join(root, "vendor");

  await mkdir(vendorDir, { recursive: true });
  process.env.SKIP_BINARY_DOWNLOAD = "1";

  try {
    await assert.rejects(
      () => install({ vendorDir, version }),
      /is missing or still a placeholder/,
    );
  } finally {
    if (previousSkip === undefined) {
      delete process.env.SKIP_BINARY_DOWNLOAD;
    } else {
      process.env.SKIP_BINARY_DOWNLOAD = previousSkip;
    }
    await rm(root, { recursive: true, force: true });
  }
});

test("wraps placeholder inspection failures when skip env is used", async () => {
  const previousSkip = process.env.SKIP_BINARY_DOWNLOAD;
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-skip-placeholder-error-"));
  const vendorDir = join(root, "vendor");
  const destination = join(vendorDir, binName);

  await mkdir(destination, { recursive: true });

  process.env.SKIP_BINARY_DOWNLOAD = "1";

  try {
    await assert.rejects(
      () => install({ baseUrl: assetBaseUrl(root), vendorDir, version }),
      /placeholder/,
    );
  } finally {
    if (previousSkip === undefined) {
      delete process.env.SKIP_BINARY_DOWNLOAD;
    } else {
      process.env.SKIP_BINARY_DOWNLOAD = previousSkip;
    }
    await rm(root, { recursive: true, force: true });
  }
});
