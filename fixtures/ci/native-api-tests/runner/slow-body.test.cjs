const { setTimeout: delay } = require("node:timers/promises");

test("injected slow body exceeds the native timing gate", async () => {
  const started = performance.now();
  await delay(5_200);
  expect(performance.now() - started).toBeGreaterThanOrEqual(5_000);
});
