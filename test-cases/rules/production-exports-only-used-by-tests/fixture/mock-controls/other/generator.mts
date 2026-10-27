import { vi } from "vitest";
// Captured replacement values, generator factories, and shadowed bindings stay uncertain.
vi.mock(import("../src/generator.mjs"), function* () { return {}; });
