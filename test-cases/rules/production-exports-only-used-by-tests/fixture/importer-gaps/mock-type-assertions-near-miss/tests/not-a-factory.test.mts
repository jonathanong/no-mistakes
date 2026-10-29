// The assertion is outside the mock factory.
import { vi } from "vitest";

const stub = {} as typeof import("../src/not-a-factory.mjs");
vi.mock("../src/not-a-factory.mjs", () => ({ argKey: stub }));
