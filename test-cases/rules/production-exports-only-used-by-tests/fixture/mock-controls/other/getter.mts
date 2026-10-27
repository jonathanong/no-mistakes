import { vi } from "vitest";
// Static object shape cannot hide opaque execution or an accessor.
vi.mock(import("../src/getter.mjs"), () => ({ get value() { return opaque.original; } }));
