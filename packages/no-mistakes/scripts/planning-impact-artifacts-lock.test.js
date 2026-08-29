const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { createPlanningArtifactLock } = require("../planning-impact-artifacts-lock");

const skipWindows = process.platform === "win32";

test(
  "retries a busy planning artifact lock until it is acquired",
  { skip: skipWindows },
  async () => {
    let attempts = 0;
    let released = 0;
    const acquire = createPlanningArtifactLock({
      acquirePlanningArtifactLock: async () => {
        attempts += 1;
        if (attempts < 3) throw new Error("planning artifact lock is busy");
        return 7;
      },
      releasePlanningArtifactLock: async (token) => {
        assert.equal(token, 7);
        released += 1;
      },
    });
    const release = await acquire("/private/run");
    assert.equal(attempts, 3);
    await release();
    assert.equal(released, 1);
  },
);

test("propagates non-busy planning artifact lock errors", { skip: skipWindows }, async () => {
  let attempts = 0;
  const acquire = createPlanningArtifactLock({
    acquirePlanningArtifactLock: async () => {
      attempts += 1;
      throw new Error("permission denied");
    },
    releasePlanningArtifactLock: async () => {
      assert.fail("must not release after a non-busy acquire error");
    },
  });
  await assert.rejects(acquire("/private/run"), /permission denied/);
  assert.equal(attempts, 1);
});
