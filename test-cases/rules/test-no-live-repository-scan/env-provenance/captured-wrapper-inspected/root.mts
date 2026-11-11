import fs from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { it } from "vitest";
async function withFixture() {
  const root = await fs.mkdtemp(join(tmpdir(), "owned-"));
  const originalReaddir = fs.readdir;
  fs.readdir = (...args) => {
    if (args[0] === root) throw new Error("fixture failure");
    return originalReaddir(...args);
  };
  try {
    await fs.readdir(root);
  } finally {
    fs.readdir = originalReaddir;
  }
}
it("scan", withFixture);
