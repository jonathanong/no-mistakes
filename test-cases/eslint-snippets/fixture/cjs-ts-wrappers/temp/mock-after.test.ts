import { vi } from "vitest";

const spec = "node:fs";
const loaded = require(spec);
loaded.writeFileSync("/tmp/a.txt", "a");
vi.mock("node:fs");
