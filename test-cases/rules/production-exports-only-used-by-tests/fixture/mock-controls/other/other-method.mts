import { vi } from "vitest";
// This callee or replacement shape cannot establish static mock isolation.
vi.other(import("../src/other-method.mjs"), () => ({}));
