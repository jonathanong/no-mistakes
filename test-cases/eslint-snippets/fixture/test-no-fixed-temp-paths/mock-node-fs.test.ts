import fs from "node:fs";
import { spawn } from "node:child_process";

vi.mock("node:fs");
fs.readFileSync("/tmp/mock-file");
spawn("tool", ["/tmp/file-ignored-with-mock"]);
