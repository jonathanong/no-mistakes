import { vi } from "vitest";
import "./setup.mjs";
import { scan } from "./scan.mjs";
vi.mock("./setup.mjs", () => ({}));
scan();
