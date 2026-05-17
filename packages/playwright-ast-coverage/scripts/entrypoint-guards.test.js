const assert = require("node:assert/strict");
const { EventEmitter } = require("node:events");
const { readFile } = require("node:fs/promises");
const { createRequire } = require("node:module");
const { join } = require("node:path");
const vm = require("node:vm");

const root = join(__dirname, "..", "..", "..");
const realRequire = createRequire(__filename);

async function runEntrypoint(relativePath, requireMock) {
  const filename = join(root, relativePath);
  const source = await readFile(filename, "utf8");
  const module = { exports: {} };
  const sandboxRequire = (id) => {
    if (requireMock && Object.hasOwn(requireMock, id)) {
      return requireMock[id];
    }
    if (id.startsWith(".")) {
      return realRequire(join(filename, "..", id));
    }
    return realRequire(id);
  };
  sandboxRequire.main = module;
  vm.runInNewContext(
    source,
    {
      __dirname: join(filename, ".."),
      console: { error() {}, log() {} },
      module,
      process: {
        argv: ["node", filename],
        exit() {},
        platform: "linux",
        env: {},
      },
      queueMicrotask,
      require: sandboxRequire,
    },
    { filename },
  );
  await new Promise((resolve) => setImmediate(resolve));
  return module.exports;
}

function childProcessMock() {
  return {
    spawn() {
      return new EventEmitter();
    },
  };
}

function coreMock() {
  return {
    install: async () => "/tmp/native-bin",
  };
}

test("package entrypoint guards execute their CLI path", async () => {
  for (const relativePath of [
    "packages/no-mistakes/bin/no-mistakes.js",
    "packages/next-to-fetch/bin/next-to-fetch.js",
    "packages/queue-ast-hop/bin/queue-ast-hop.js",
    "packages/react-traits/bin/react-traits.js",
    "packages/server-ast-routes/bin/server-ast-routes.js",
  ]) {
    const exports = await runEntrypoint(relativePath, {
      "node:child_process": childProcessMock(),
    });
    assert.equal(typeof exports.run, "function");
  }
});

test("install entrypoint guards execute their main path", async () => {
  for (const relativePath of [
    "packages/no-mistakes/scripts/install.js",
    "packages/next-to-fetch/scripts/install.js",
    "packages/queue-ast-hop/scripts/install.js",
    "packages/react-traits/scripts/install.js",
    "packages/server-ast-routes/scripts/install.js",
  ]) {
    const exports = await runEntrypoint(relativePath, {
      "no-mistakes-core": coreMock(),
    });
    assert.equal(typeof exports.main, "function");
  }
});

test("playwright package entrypoint guards execute", async () => {
  const binExports = await runEntrypoint(
    "packages/playwright-ast-coverage/bin/playwright-ast-coverage.js",
    {
      "./cli": { run: () => 0 },
    },
  );
  assert.equal(typeof binExports.main, "function");

  const installExports = await runEntrypoint(
    "packages/playwright-ast-coverage/scripts/install.js",
    {
      "no-mistakes-core": {
        ...coreMock(),
        assetName: (bin, version, target) => `${bin}-${version}-${target}`,
        releaseBaseUrl: () => "https://example.test",
        unsupportedPlatformMessage: () => "unsupported",
      },
    },
  );
  assert.equal(typeof installExports.main, "function");
});
