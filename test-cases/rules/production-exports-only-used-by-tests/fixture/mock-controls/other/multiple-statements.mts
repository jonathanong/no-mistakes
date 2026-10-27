import { vi } from "vitest";
// This callee or replacement shape cannot establish static mock isolation.
vi.mock(import("../src/multiple-statements.mjs"), () => { const value = 1; return { value }; });
