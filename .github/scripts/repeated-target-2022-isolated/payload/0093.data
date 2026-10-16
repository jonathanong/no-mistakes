"use strict";

const native = require(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH);

async function main() {
  const token = await native.acquirePlanningArtifactLock(process.argv[2]);
  process.send("locked");
  process.once("message", async (message) => {
    if (message !== "release") return;
    await native.releasePlanningArtifactLock(token);
    process.exit(0);
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
