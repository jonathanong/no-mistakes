import { execFile, execFileSync, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { beforeAll, describe, expect, it } from "vitest";

const repoRoot = resolve(import.meta.dirname, "..");

it("allows reading a script that the test executes via spawnSync", () => {
  const scriptPath = join(repoRoot, "scripts/check.sh");
  const text = readFileSync(scriptPath, "utf8");
  const result = spawnSync("bash", [scriptPath], { encoding: "utf8" });
  expect(result.status).toBe(0);
  expect(text).toContain("set -euo pipefail");
});

it("allows reading a script executed directly via execFile", () => {
  const scriptPath = join(repoRoot, "scripts/run.sh");
  const text = readFileSync(scriptPath, "utf8");
  execFile(scriptPath, ["--dry-run"], () => {});
  expect(text).toContain("#!/usr/bin/env bash");
});

it("allows equivalent static paths between readFileSync and execFileSync", () => {
  const text = readFileSync(join(repoRoot, "scripts/lint.mjs"), "utf8");
  execFileSync(process.execPath, [join(repoRoot, "scripts/lint.mjs")]);
  expect(text).toContain("process.exitCode");
});

describe("suite with beforeAll execution", () => {
  const scriptPath = join(repoRoot, "scripts/release.sh");
  beforeAll(() => {
    spawnSync("bash", ["-n", scriptPath]);
  });

  it("allows reading script executed in beforeAll", () => {
    const text = readFileSync(scriptPath, "utf8");
    expect(text).toContain("set -e");
  });
});
