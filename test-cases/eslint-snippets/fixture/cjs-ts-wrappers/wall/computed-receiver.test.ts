import { expect } from "vitest";

declare const spec: string;
declare const key: string;
const clocks = require(spec);
expect(clocks[key].now() - 0).toBeLessThan(200);
