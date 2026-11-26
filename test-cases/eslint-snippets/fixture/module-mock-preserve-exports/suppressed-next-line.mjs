import { vi } from "vitest";

// no-mistakes-disable-next-line module-mock-preserve-exports -- retained probe
vi.mock("./local", () => ({ charge() {} }));
