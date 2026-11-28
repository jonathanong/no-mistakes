import { expect } from "vitest";
const spec = "node:perf_hooks";
const clock = require(spec);
const start = 0;
expect(clock["now"]() - start).toBeLessThan(200);
expect(clock.now() - start)["toBeLessThan"](200);
