import { vi } from "vitest";

vi.mock("node:fs");
const fs = require("node:fs");
const alias = fs;
{
  const fs = alias;
  fs.writeFileSync("/tmp/result", "");
}
