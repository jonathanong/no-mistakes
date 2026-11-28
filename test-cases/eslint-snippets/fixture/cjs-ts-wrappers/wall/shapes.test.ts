import { expect } from "vitest";
import * as ph from "node:perf_hooks";

const start = 1;
expect(ph.performance.now() - ph.performance.mark()).toBeLessThan(10);
expect(ph.performance.mark() - start).toBeGreaterThan(0);
ph.performance.now().toBeLessThan(1);
const elapsed = 1;
elapsed.toBeLessThan(10);
