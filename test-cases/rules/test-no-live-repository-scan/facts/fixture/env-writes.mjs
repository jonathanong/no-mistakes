// Every shape of process.env write the path-provenance pass records or refuses.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { beforeAll, describe, it, vi } from "vitest";

const dir = mkdtempSync(tmpdir());
const key = String(Math.random());
process.env.A = dir;
process.env["B"] = dir;
process.env.C ??= dir;
delete process.env.D;
vi.stubEnv("E", dir);
if (key) {
  process.env.F = dir;
}
{
  process.env.G = dir;
}
function setup() {
  process.env.H = dir;
}
setup();
describe("suite", () => {
  beforeAll(() => {
    process.env.I = dir;
  });
  it("runs", () => {
    process.env.J = dir;
  });
});
const nested = (process.env.K = dir);
vi.stubEnv("L");
it("function form", function () {});
