import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, it, vi } from "vitest";
import { scanOwnedArgument, scanOwnedEnvironment } from "../src/scans.mjs";

const roots: string[] = [];
afterEach(() => {
  vi.unstubAllEnvs();
  for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true });
});

it("param", () => {
  const root = mkdtempSync(join(tmpdir(), "owned-root-parameter-"));
  roots.push(root);
  writeFileSync(join(root, "fixture.txt"), "");
  scanOwnedArgument(root);
});

it("env", () => {
  const root = mkdtempSync(join(tmpdir(), "owned-root-environment-"));
  roots.push(root);
  writeFileSync(join(root, "fixture.txt"), "");
  vi.stubEnv("OWNED_SCAN_ROOT", process.cwd());
  scanOwnedEnvironment();
});
