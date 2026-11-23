import { performance as clock } from "node:perf_hooks";
import { performance } from "perf_hooks";
import { performance as customClock } from "./clock";

expect(clock.now() - start).toBeLessThan(200);
expect(performance.now() - start).toBeLessThan(200);
expect(customClock.now() - start).toBeLessThan(200);
function check(clock) {
  expect(clock.now() - start).toBeLessThan(200);
}
