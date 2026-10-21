import { vi } from "vitest";

const mock = vi.mock as typeof vi.mock;
mock("./internal", () => ({ charge() {} }));
