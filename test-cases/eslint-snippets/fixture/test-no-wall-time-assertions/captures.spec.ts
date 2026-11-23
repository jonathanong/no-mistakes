import { expect as verify } from "vitest";

test("asserts on captured elapsed time", async () => {
  const start = performance.now();
  await run();
  const end = performance.now();
  const elapsed = (end - start) satisfies number;
  const check = verify;
  {
    const observed = elapsed;
    check(observed).not.toBeGreaterThanOrEqual(200);
  }
  verify(end - start)["toBeLessThanOrEqual"](200);
});
