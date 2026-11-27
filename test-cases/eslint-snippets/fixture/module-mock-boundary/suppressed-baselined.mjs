import { vi } from "vitest";

vi.mock("./local", () => ({ charge() {} })); // no-mistakes-disable-line module-mock-boundary -- baselined call still counts
