import { vi } from "vitest";

vi.mock("../src/api.mjs", () => ({}) as typeof import("../src/api.mjs"));
