import { vi } from "vitest";
const target = "../src/runtime.mjs";
vi.doMock(import(target), () => ({ selected: 0 })); console.log(await import(target));
