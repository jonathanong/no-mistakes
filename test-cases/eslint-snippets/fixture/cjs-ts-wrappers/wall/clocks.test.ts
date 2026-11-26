import { expect } from "vitest";
import * as ph from "node:perf_hooks";
import phEquals = require("node:perf_hooks");
const { performance: perf } = ph;
const start = 1;
expect(ph.performance.now() - start).toBeLessThan(10);
expect(phEquals.performance.now() - start).toBeLessThan(10);
expect(perf.now() - start).toBeLessThan(10);
