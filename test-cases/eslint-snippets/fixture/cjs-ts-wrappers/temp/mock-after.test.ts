import { vi } from "vitest";

const spec = "node:fs";
const loaded = require(spec);
loaded.writeFileSync("/tmp/a.txt", "a");
const fs = require("node:fs");
fs.writeFileSync("/tmp/b.txt", "b");
vi.mock("node:fs");
