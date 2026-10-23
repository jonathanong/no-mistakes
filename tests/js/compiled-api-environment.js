const { dirname, join } = require("node:path");

function configureCompiledApiEnvironment({ platform = process.platform, env = process.env } = {}) {
  const addon = env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
  if (platform === "linux" || !addon?.endsWith(".node")) return;

  // Cargo's existing test marker selects a PID-specific invocation lock on platforms
  // without Linux runtime directories. Same-process native calls still share a lock.
  env["CARGO_BIN_EXE_no-mistakes"] =
    env.NO_MISTAKES_TEST_CLI_PATH ||
    join(dirname(addon), platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
}

module.exports = { configureCompiledApiEnvironment };
