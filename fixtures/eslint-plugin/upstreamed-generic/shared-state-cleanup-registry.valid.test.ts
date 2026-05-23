import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const testDirs: string[] = [];

afterEach(async () => {
  for (const dir of testDirs) {
    await rm(dir, { recursive: true, force: true });
  }
  testDirs.length = 0;
});

test("creates a temp dir", async () => {
  const dir = await mkdtemp(join(tmpdir(), "no-mistakes-"));
  testDirs.push(dir);
});

