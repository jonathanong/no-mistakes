import { vi } from "vitest";
import "./setup.mjs";
import { scan } from "./scan.mjs";
vi.mock(import("./setup.mjs"), () => ({}));
scan();
