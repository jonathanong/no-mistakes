import fs from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { it } from "vitest";
async function withFixture() {
  const root = await fs.mkdtemp(join(tmpdir(), "owned-"));
  const originalReaddir = fs.readdir;
  fs.readdir = (...args) => {
    scanOtherRoot();
    return originalReaddir(...args);
  };
  try {
    await fs.readdir(root);
  } finally {
    fs.readdir = originalReaddir;
  }
}
it("scan", withFixture);
function scanOtherRoot() { return fs.readdir(process.cwd()); }
