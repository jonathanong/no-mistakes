import * as fs from "node:fs/promises";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { vi } from "vitest";
const root = mkdtempSync(join(tmpdir(), "mock-forwarded-"));
const CANONICAL = process.cwd();
const originalReaddir = fs.readdir;
vi.spyOn(fs, "readdir").mockImplementation((...args) => {
  args[0] = root;
  return originalReaddir(...args);
});
