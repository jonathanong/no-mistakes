import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const testDirs: string[] = [];
const activeDirs = new Set<string>();
let cleanupCount = 0;

afterEach(async () => {
  for (const dir of testDirs) {
    await rm(dir, { recursive: true, force: true });
  }
  testDirs.length = 0;
  activeDirs.clear();
  cleanupCount = 0;
  cleanupCount++;
});

test("creates a temp dir", async () => {
  const dir = await mkdtemp(join(tmpdir(), "no-mistakes-"));
  testDirs.push(dir);
  activeDirs.add(dir);
});
