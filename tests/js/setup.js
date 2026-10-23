const { join } = require("node:path");
const { mkdtempSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { configureCompiledApiEnvironment } = require("./compiled-api-environment.js");

configureCompiledApiEnvironment();

if (
  process.platform === "linux" &&
  process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node")
) {
  // Forked test files need distinct native invocation locks; their own children inherit this one.
  const runtime = mkdtempSync(join(tmpdir(), "no-mistakes-api-runtime-"));
  process.env.XDG_RUNTIME_DIR = runtime;
  process.once("exit", () => rmSync(runtime, { recursive: true, force: true }));
}

process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH ||= join(
  __dirname,
  "..",
  "..",
  "fixtures",
  "napi",
  "test-addon.js",
);
