import { vi } from "vitest";
const target = "../src/same-line.mjs";
vi.doMock(import(target), () => ({ selected: 0 })); console.log((await import(target)).selected);
