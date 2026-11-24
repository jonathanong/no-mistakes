import { execFile } from "node:child_process";
import { readFileSync } from "node:fs";
import { tmpdir } from "node:os";

readFileSync(new URL("file:///tmp/result.json"));
readFileSync(`${tmpdir()}/result.json`);
readFileSync(tmpdir() + "/result.json");

vi.doMock("node:fs");
jest.doMock("node:fs");
readFileSync("/tmp/result.json");
execFile("tool", ["/tmp/example description"]);
