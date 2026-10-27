import { vi } from "vitest";
// Captured replacement values, generator factories, and shadowed bindings stay uncertain.
function replace(vi: unknown) { vi.mock(import("../src/shadowed.mjs"), () => ({ value: 2 })); }
