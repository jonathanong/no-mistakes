import { expect as check, vi as timers } from "vitest";

test("checks a virtual delay", () => {
  check(performance.now() - start).toBeLessThan(200);
});
timers["useFakeTimers"]();
