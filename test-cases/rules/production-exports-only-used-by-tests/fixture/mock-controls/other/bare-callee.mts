import { vi } from "vitest";
// This callee or replacement shape cannot establish static mock isolation.
mock(import("../src/bare-callee.mjs"), () => ({}));
