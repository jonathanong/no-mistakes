import { vi } from "vitest";
// This callee or replacement shape cannot establish static mock isolation.
api.vi.mock(import("../src/nested-callee.mjs"), () => ({}));
