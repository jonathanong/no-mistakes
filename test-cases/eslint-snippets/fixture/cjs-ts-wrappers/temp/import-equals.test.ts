import { vi } from "vitest";
import fs = require("node:fs");

vi.mock("node:fs");
fs.writeFileSync("/tmp/result", "");
