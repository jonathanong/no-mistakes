import { vi } from "vitest";

vi.mock("./local", () => ({ charge() {} }));
