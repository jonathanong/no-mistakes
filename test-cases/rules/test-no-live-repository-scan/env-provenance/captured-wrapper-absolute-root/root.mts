import fs from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { it } from "vitest";
async function withFixture() {
  const root = await fs.mkdtemp(join(tmpdir(), "owned-"));
  const originalReaddir = fs.readdir;
  fs.readdir = (...args) => {
    return originalReaddir(...args);
  };
  try {
    await fs.readdir(resolve(root, "/unowned"));
  } finally {
    fs.readdir = originalReaddir;
  }
}
it("scan", withFixture);
