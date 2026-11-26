const assert = require("node:assert/strict");
const api = require("../../../packages/no-mistakes/index.js");
const native = require(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH);

async function run() {
  const { lockFile } = await api.lockStatus();
  const token = await native.acquirePlanningArtifactLock(lockFile);
  const options = { plan: process.argv[2], failOnLock: true };
  try {
    for (const operation of ["testsComment", "testsGraphMermaid"]) {
      await assert.rejects(api[operation](options), (error) => {
        assert.equal(error.code, "NO_MISTAKES_LOCK_BUSY");
        assert.equal(error.details.kind, "lock_busy");
        assert.doesNotMatch(error.message, /no-mistakes-error-json/);
        return true;
      });
    }
  } finally {
    await native.releasePlanningArtifactLock(token);
  }
  process.stdout.write("2 structured lock errors\n");
}

run().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
