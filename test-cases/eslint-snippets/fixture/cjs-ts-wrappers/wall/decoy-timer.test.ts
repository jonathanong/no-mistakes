import { expect } from "vitest";

const start = 1;
function clock() {}
clock.useFakeTimers();
expect(Date.now() - start).toBeLessThan(100);
