import { expect, vi } from "vitest";

const spec = "node:perf_hooks";
const ph = require(spec);
const start = 1;
expect(ph.performance.now() - start).toBeLessThan(10);
vi.useFakeTimers();
