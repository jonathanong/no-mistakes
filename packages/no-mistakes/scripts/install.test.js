const assert = require("node:assert/strict");
const { mkdir, rm, writeFile } = require("node:fs/promises");
const { join } = require("node:path");

const core = require("./install/index");
const { main } = require("./install");
const {
  isGlibc,
  parseChecksum,
  platformTarget,
  install,
  supportedGlibc,
  unsupportedPlatformMessage,
} = core;

const binName = "no-mistakes";
const repository = "jonathanong/no-mistakes";

function assetName(version, target) {
  return core.assetName({ binName, version, target });
}

function releaseBaseUrl(version) {
  return core.releaseBaseUrl(repository, version, "NO_MISTAKES_RELEASE_BASE_URL");
}

function glibcReport(version = "2.39") {
  return {
    getReport() {
      return { header: { glibcVersionRuntime: version } };
    },
  };
}

function muslReport() {
  return {
    getReport() {
      return { header: {} };
    },
  };
}

test("maps supported platforms to Rust targets", () => {
  assert.equal(platformTarget("darwin", "x64"), "x86_64-apple-darwin");
  assert.equal(platformTarget("darwin", "arm64"), "aarch64-apple-darwin");
  assert.equal(platformTarget("win32", "x64"), "x86_64-pc-windows-msvc");
  assert.equal(platformTarget("linux", "x64", glibcReport("2.35")), "x86_64-unknown-linux-gnu");
  assert.equal(platformTarget("linux", "arm64", glibcReport("2.39")), "aarch64-unknown-linux-gnu");
});

test("installer main succeeds when binary download is skipped", async () => {
  const calls = [];
  await main(async (...args) => {
    calls.push(args);
    return `/tmp/${args[2].destinationName}`;
  });
  assert.equal(calls.length, 2);
  assert.equal(calls[0][2].destinationName, "no-mistakes");
  assert.equal(calls[1][0], "no-mistakes-napi");
  assert.equal(calls[1][2].destinationName, "no-mistakes.node");
  assert.equal(calls[1][2].assetExtension, ".node");
});

test("installer main reports failures", async () => {
  const exits = [];
  const errors = [];
  await main(
    async () => {
      throw new Error("install failed");
    },
    { exit: (code) => exits.push(code) },
    { log() {}, error: (message) => errors.push(message) },
  );
  assert.deepEqual(exits, [1]);
  assert.deepEqual(errors, ["install failed"]);
  await main(
    async () => {
      throw "string failed";
    },
    { exit: (code) => exits.push(code) },
    { log() {}, error: (message) => errors.push(message) },
  );
  assert.deepEqual(errors.slice(-1), ["string failed"]);
});

test("rejects unsupported platform targets", () => {
  assert.equal(platformTarget("linux", "x64", muslReport()), null);
  assert.equal(platformTarget("linux", "x64", glibcReport("2.31")), null);
  assert.equal(platformTarget("freebsd", "x64"), null);
  assert.equal(platformTarget("win32", "arm64"), null);
});

test("checks minimum glibc version", () => {
  assert.equal(supportedGlibc(glibcReport("2.34")), false);
  assert.equal(supportedGlibc(glibcReport("2.35")), true);
  assert.equal(supportedGlibc(glibcReport("3.0")), true);
  assert.equal(supportedGlibc(glibcReport("nope")), false);
  assert.equal(supportedGlibc(muslReport()), false);
  assert.equal(isGlibc(glibcReport("2.39")), true);
  assert.equal(isGlibc(muslReport()), false);
});

test("formats release asset names", () => {
  assert.equal(
    assetName("1.2.3", "x86_64-unknown-linux-gnu"),
    "no-mistakes-v1.2.3-x86_64-unknown-linux-gnu",
  );
  assert.equal(
    assetName("1.2.3", "x86_64-pc-windows-msvc"),
    "no-mistakes-v1.2.3-x86_64-pc-windows-msvc.exe",
  );
  assert.equal(
    core.assetName("no-mistakes", "1.2.3", "x86_64-unknown-linux-gnu"),
    "no-mistakes-v1.2.3-x86_64-unknown-linux-gnu",
  );
  assert.equal(
    core.assetName({
      binName: "no-mistakes-napi",
      version: "1.2.3",
      target: "x86_64-pc-windows-msvc",
      assetExtension: ".node",
    }),
    "no-mistakes-napi-v1.2.3-x86_64-pc-windows-msvc.node",
  );
});

test("assetName validates positional and object-call inputs", () => {
  assert.throws(() => core.assetName("no-mistakes", "1.2.3", 1), TypeError);
  assert.throws(() => core.assetName("no-mistakes", 1, "x86_64-unknown-linux-gnu"), TypeError);
  assert.throws(
    () =>
      core.assetName({
        binName: "no-mistakes",
        version: "1.2.3",
        target: 1,
      }),
    TypeError,
  );
});

test("formats release base URLs", () => {
  const previous = process.env.NO_MISTAKES_RELEASE_BASE_URL;
  try {
    delete process.env.NO_MISTAKES_RELEASE_BASE_URL;
    assert.equal(
      releaseBaseUrl("1.2.3"),
      "https://github.com/jonathanong/no-mistakes/releases/download/v1.2.3",
    );
    process.env.NO_MISTAKES_RELEASE_BASE_URL = "https://example.test/releases";
    assert.equal(releaseBaseUrl("1.2.3"), "https://example.test/releases");
    assert.equal(require("../package.json").version, require("../package.json").version);
  } finally {
    if (previous === undefined) {
      delete process.env.NO_MISTAKES_RELEASE_BASE_URL;
    } else {
      process.env.NO_MISTAKES_RELEASE_BASE_URL = previous;
    }
  }
});

test("supports legacy install overloads and unsupported platform overload", async () => {
  const previousSkip = process.env.SKIP_BINARY_DOWNLOAD;
  const vendor = join(__dirname, "..", "vendor");
  const executable = join(__dirname, "..", "bin", "no-mistakes");
  const custom = join(vendor, "custom-bin");
  const customDestination = join(vendor, "custom-dest");

  try {
    await mkdir(vendor, { recursive: true });
    await writeFile(custom, "custom");
    await writeFile(customDestination, "custom destination");

    assert.equal(executable.endsWith("bin/no-mistakes"), true);
    process.env.SKIP_BINARY_DOWNLOAD = "1";
    assert.equal(
      await install("custom-bin", "owner/repo", {
        destinationName: "custom-dest",
        target: "x86_64-unknown-linux-gnu",
        vendorDir: vendor,
        version: "1.0.0",
      }),
      customDestination,
    );
    delete process.env.SKIP_BINARY_DOWNLOAD;
    assert.equal(
      await install("custom-bin", "owner/repo", {
        checkExisting: true,
        target: "x86_64-unknown-linux-gnu",
        vendorDir: vendor,
        version: "1.0.0",
      }),
      custom,
    );
    assert.match(
      unsupportedPlatformMessage(binName, "linux", "x64", { getReport: () => ({ header: {} }) }),
      /glibc/,
    );
  } finally {
    if (previousSkip === undefined) {
      delete process.env.SKIP_BINARY_DOWNLOAD;
    } else {
      process.env.SKIP_BINARY_DOWNLOAD = previousSkip;
    }
    await rm(vendor, { recursive: true, force: true });
  }
});

test("parses sha256 files with or without filenames", () => {
  const hash = "a".repeat(64);
  assert.equal(parseChecksum(`not-a-hash binary\n${hash} binary\n`, "binary"), hash);
  assert.equal(parseChecksum(`${hash}  binary\n`, "binary"), hash);
  assert.equal(parseChecksum(`${hash}\n`, "binary"), hash);
  assert.equal(parseChecksum(`${hash} *binary\n`, "binary"), hash);
  assert.equal(parseChecksum(`${hash}  /tmp/release/binary\n`, "binary"), hash);
  assert.throws(() => parseChecksum(`${hash} other\n`, "binary"), /No SHA-256 checksum/);

  // uppercase hashes
  const upperHash = hash.toUpperCase();
  assert.equal(parseChecksum(`${upperHash} binary\n`, "binary"), hash);

  // CRLF line endings
  assert.equal(parseChecksum(`${hash} binary\r\n`, "binary"), hash);

  // Empty lines and whitespace
  assert.equal(parseChecksum(`\n   \n${hash} binary\n\n`, "binary"), hash);
  assert.equal(parseChecksum(`  ${hash} binary  \n`, "binary"), hash);

  // Invalid hashes
  const shortHash = "a".repeat(63);
  const longHash = "a".repeat(65);
  const invalidHex = "z".repeat(64);
  assert.throws(() => parseChecksum(`${shortHash} binary\n`, "binary"), /No SHA-256 checksum/);
  assert.throws(() => parseChecksum(`${longHash} binary\n`, "binary"), /No SHA-256 checksum/);
  assert.throws(() => parseChecksum(`${invalidHex} binary\n`, "binary"), /No SHA-256 checksum/);

  // Multiline ignoring other files and finding the correct one
  const otherHash = "b".repeat(64);
  const multilineText = `
${otherHash}  other_binary
${hash}  binary
${otherHash}  another_binary
`;
  assert.equal(parseChecksum(multilineText, "binary"), hash);
});
