// no-mistakes-disable-file module-mock-preserve-exports -- retained probe
import { vi } from "vitest";

vi.mock("./local", () => ({ charge() {} }));
