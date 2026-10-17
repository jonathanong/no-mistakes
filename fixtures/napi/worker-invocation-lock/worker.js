const { mkdirSync } = require("node:fs");
const { join } = require("node:path");

const [mode, initialize] = process.argv.slice(2);
process.once("SIGTERM", () => process.exit(0));
if (initialize === "setup") require("../../../tests/js/setup.js");

async function main() {
  const runtime = process.env.XDG_RUNTIME_DIR;
  if (mode === "environment") {
    process.send({ runtime });
    process.disconnect();
    return;
  }
  if (mode === "hold") {
    const native = require(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH);
    const directory = join(runtime, "no-mistakes");
    mkdirSync(directory, { recursive: true });
    const token = await native.acquirePlanningArtifactLock(join(directory, "invocation.lock"));
    process.once("message", async () => {
      await native.releasePlanningArtifactLock(token);
      process.disconnect();
    });
    process.send({ runtime });
    return;
  }
  const api = require("../../../packages/no-mistakes/index.js");
  try {
    await api.dependencies({
      root: join(__dirname, "../real-addon-dependencies"),
      files: ["entry.ts"],
      failOnLock: true,
    });
    process.send({ runtime, success: true });
  } catch (error) {
    process.send({ runtime, error: error.message });
  }
  process.disconnect();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
  process.disconnect();
});
