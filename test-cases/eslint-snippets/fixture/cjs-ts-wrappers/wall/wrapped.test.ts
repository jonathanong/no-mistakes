import * as v from "vitest";
const start = 1;
(v as typeof v).expect(Date.now() - start).toBeLessThan(100);
