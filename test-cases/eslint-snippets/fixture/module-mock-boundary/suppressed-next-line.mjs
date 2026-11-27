import { vi } from "vitest";

// no-mistakes-disable-next-line module-mock-boundary -- retained probe
vi.mock("./local", () => ({ charge() {} }));
