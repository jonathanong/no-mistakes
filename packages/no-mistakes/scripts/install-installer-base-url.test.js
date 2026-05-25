const { createHash } = require("node:crypto");
const { createServer } = require("node:http");
const assert = require("node:assert/strict");
const { mkdir, mkdtemp, rm } = require("node:fs/promises");
const { join } = require("node:path");
const { tmpdir } = require("node:os");
const core = require("./install/index");

const repositoryName = "jonathanong/no-mistakes";
const version = "9.8.7";
const binName = "no-mistakes";

function install(options) {
  return core.install("no-mistakes", repositoryName, {
    destinationName: "no-mistakes",
    envVar: "NO_MISTAKES_RELEASE_BASE_URL",
    ...options,
  });
}

function assetName(target) {
  return core.assetName(binName, version, target);
}

test("rejects malformed release base URLs", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-bad-url-"));
  const vendorDir = join(root, "vendor");

  await mkdir(vendorDir, { recursive: true });

  try {
    await assert.rejects(
      () =>
        install({
          baseUrl: "https://",
          target: "x86_64-unknown-linux-gnu",
          vendorDir,
          version,
        }),
      /Invalid release base URL: https:\/\/.*/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("normalizes trailing slash in base URLs", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-base-url-trailing-slash-"));
  const vendorDir = join(root, "vendor");
  await mkdir(vendorDir, { recursive: true });

  const target = "x86_64-unknown-linux-gnu";
  const asset = assetName(target);
  const content = Buffer.from("binary");
  const checksum = createHash("sha256").update(content).digest("hex");
  let sawDoubleSlash = false;

  const server = createServer((request, response) => {
    if (request.url.includes("//")) {
      sawDoubleSlash = true;
    }
    if (request.url === `/${asset}`) {
      response.writeHead(200);
      response.end(content);
      return;
    }
    if (request.url === `/${asset}.sha256`) {
      response.writeHead(200);
      response.end(`${checksum}  ${asset}\n`);
      return;
    }
    response.writeHead(404);
    response.end("not found");
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await install({
      baseUrl: `http://127.0.0.1:${address.port}/`,
      target,
      vendorDir,
      version,
    });
    assert.equal(sawDoubleSlash, false);
  } finally {
    await new Promise((resolve) => server.close(resolve));
    await rm(root, { recursive: true, force: true });
  }
});

test("rejects insecure github release URLs", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-bad-protocol-"));
  const vendorDir = join(root, "vendor");

  await mkdir(vendorDir, { recursive: true });

  try {
    await assert.rejects(
      () =>
        install({
          baseUrl: `http://github.com/${repositoryName}/releases/download/v${version}`,
          target: "x86_64-unknown-linux-gnu",
          vendorDir,
          version,
        }),
      /Expected https: protocol/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("allows mixed-case repository segments on github release URLs", () => {
  assert.doesNotThrow(() => {
    core.validateReleaseBaseUrl(
      "https://github.com/JonathanOnG/No-Mistakes/releases/download/v9.8.7",
      repositoryName,
    );
  });
});

test("allows release-asset redirect hosts", () => {
  assert.doesNotThrow(() => {
    core.validateReleaseBaseUrl(
      "https://release-assets.githubusercontent.com/example",
      repositoryName,
      {
        enforcePath: false,
      },
    );
  });
});

test("allows local testing hosts with enforcePath false", () => {
  assert.doesNotThrow(() => {
    core.validateReleaseBaseUrl(
      "https://127.0.0.1/jonathanong/no-mistakes/releases/download/v9.8.7",
      repositoryName,
      {
        enforcePath: false,
      },
    );
  });
  assert.doesNotThrow(() => {
    core.validateReleaseBaseUrl("http://example.test/example/path", repositoryName, {
      enforcePath: false,
    });
  });
});

test("allows local testing hosts during enforcePath validation", () => {
  assert.doesNotThrow(() => {
    core.validateReleaseBaseUrl("http://127.0.0.1/custom/path", repositoryName);
  });
});

test("requires github.com when enforcing base path", () => {
  assert.throws(
    () =>
      core.validateReleaseBaseUrl(
        "https://release-assets.githubusercontent.com/example",
        repositoryName,
      ),
    /expected base URL host github\.com/,
  );
});

test("enforces allowed hosts during redirect-mode validation", () => {
  assert.throws(
    () =>
      core.validateReleaseBaseUrl(
        "https://example.com/jonathanong/no-mistakes/releases/download/v9.8.7",
        repositoryName,
        {
          enforcePath: false,
        },
      ),
    /Allowed hosts are: github\.com, 127\.0\.0\.1, example\.test, \*\.githubusercontent\.com/,
  );
});

test("allows file URLs without host", () => {
  assert.doesNotThrow(() => {
    core.validateReleaseBaseUrl("file:///tmp/no-mistakes", "jonathanong/no-mistakes");
  });
});

test("rejects non-canonical file URLs", () => {
  assert.throws(
    () => core.validateReleaseBaseUrl("file:/tmp/no-mistakes", repositoryName),
    /canonical 'file:\//,
  );
});

test("rejects file URLs with hosts", () => {
  assert.throws(
    () => core.validateReleaseBaseUrl("file://attacker/share/no-mistakes", repositoryName),
    /canonical 'file:\//,
  );
});

test("rejects non-official github release paths", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-bad-path-"));
  const vendorDir = join(root, "vendor");

  await mkdir(vendorDir, { recursive: true });

  try {
    await assert.rejects(
      () =>
        install({
          baseUrl: `https://github.com/${repositoryName}.wrong/release/download/v${version}`,
          target: "x86_64-unknown-linux-gnu",
          vendorDir,
          version,
        }),
      /expected base URL prefix .*\/jonathanong\/no-mistakes\/releases\/download/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("rejects disallowed github hosts", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-bad-host-"));
  const vendorDir = join(root, "vendor");

  await mkdir(vendorDir, { recursive: true });

  try {
    await assert.rejects(
      () =>
        install({
          baseUrl: "https://example.com/jonathanong/no-mistakes/releases/download/v9.8.7",
          target: "x86_64-unknown-linux-gnu",
          vendorDir,
          version,
        }),
      /expected base URL host github\.com/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
