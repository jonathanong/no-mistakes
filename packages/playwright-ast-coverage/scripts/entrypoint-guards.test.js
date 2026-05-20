const assert = require("node:assert/strict");
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

function coreMock(calls = []) {
  return {
    install: async (...args) => {
      calls.push(args);
      return "/tmp/native-bin";
    },
  };
}

test("install entrypoint guards execute their main path", async () => {
  const installers = [
    ["packages/no-mistakes/scripts/install.js", "no-mistakes"],
    ["packages/next-to-fetch/scripts/install.js", "next-to-fetch"],
    ["packages/queue-ast-hop/scripts/install.js", "queue-ast-hop"],
    ["packages/react-traits/scripts/install.js", "react-traits"],
    ["packages/server-ast-routes/scripts/install.js", "server-ast-routes"],
  ];
  for (const [relativePath, binary] of installers) {
    const calls = [];
    const exports = await runEntrypoint(relativePath, {
      "./install/index": coreMock(calls),
    });
    assert.equal(typeof exports.main, "function");
    assert.deepEqual(
      calls.map(([bin, repository]) => [bin, repository]),
      [[binary, "jonathanong/no-mistakes"]],
    );
  }
});

test("playwright install entrypoint guard executes", async () => {
  const calls = [];
  const installExports = await runEntrypoint(
    "packages/playwright-ast-coverage/scripts/install.js",
    {
      "./install/index": {
        ...coreMock(calls),
        assetName: (bin, version, target) => `${bin}-${version}-${target}`,
        releaseBaseUrl: () => "https://example.test",
        unsupportedPlatformMessage: () => "unsupported",
      },
    },
  );
  assert.equal(typeof installExports.main, "function");
  assert.deepEqual(
    calls.map(([bin, repository]) => [bin, repository]),
    [["playwright-ast-coverage", "jonathanong/no-mistakes"]],
  );
});
