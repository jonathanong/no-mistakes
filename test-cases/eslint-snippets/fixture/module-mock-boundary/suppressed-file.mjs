// no-mistakes-disable-file module-mock-boundary -- retained probe
import { vi } from "vitest";

vi.mock("./local", () => ({ charge() {} }));
