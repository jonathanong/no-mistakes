import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { it } from "vitest";
import {
  candidateSourceHash,
  fixture,
  hashSources,
  packageRoot,
} from "../bench/source-proof-prepare.mjs";

it("rejects after measurements without a distinct candidate, including explicit baselines", () => {
  const root = mkdtempSync(resolve(tmpdir(), "no-mistakes-proof-archives-"));
  try {
    const command = resolve(packageRoot, "bench/source-proof.mjs");
    const env = { ...process.env, NODE_PATH: "", NODE_OPTIONS: "" };
    // The real preparation process audits physical peer absence, independently
    // of the test runner's module-resolution hooks or parent NODE_PATH.
    const prepared = spawnSync(
      process.execPath,
      [command, "prepare", root, "before", "1", "sparse", "HEAD"],
      {
        encoding: "utf8",
        env,
      },
    );
    assert.equal(prepared.status, 0, prepared.stderr);
    assert.throws(() => candidateSourceHash(root), /before and after source archives must differ/);
    const result = spawnSync(process.execPath, [command, "measure", root, "after-control"], {
      encoding: "utf8",
      env,
    });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /before and after source archives must differ/);
    // A saved source change makes the candidate distinct; stale metadata alone
    // must not decide whether the actual archives differ.
    cpSync(
      resolve(fixture, "shared/consumers.ts"),
      resolve(root, "peers/after/src/benchmark-control.ts"),
    );
    assert.equal(candidateSourceHash(root), hashSources(resolve(root, "peers/after/src")));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
