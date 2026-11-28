import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { beforeEach, describe, expect, it } from "vitest";

const repoRoot = resolve(import.meta.dirname, "..");

it("allows default fixture directories", () => {
  expect(readFileSync("fixtures/ci.yml", "utf8")).toContain("persist-credentials: false");
  expect(readFileSync(join(import.meta.dirname, "__fixtures__", "ci.yml"), "utf8")).toContain(
    "persist-credentials: false",
  );
  expect(readFileSync(resolve(repoRoot, "test-cases/sample/config.json"), "utf8")).toContain(
    "example",
  );
  expect(readFileSync(new URL("./fixtures/ci.yml", import.meta.url), "utf8")).toContain(
    "persist-credentials: false",
  );
});

it("allows files written by the test or created in temp dirs", () => {
  const out = join(repoRoot, "generated-output.json");
  writeFileSync(out, JSON.stringify({ ok: true }));
  expect(readFileSync(out, "utf8")).toContain('"ok":true');

  const dir = mkdtempSync(join(tmpdir(), "mirror-"));
  expect(readFileSync(join(dir, "config.yml"), "utf8")).toContain("key: value");

  expect(readFileSync("/tmp/config.yml", "utf8")).toContain("key: value");
});

describe("beforeEach write covers test read", () => {
  const out = join(repoRoot, "generated-by-hook.json");
  beforeEach(() => {
    writeFileSync(out, '{"ready":true}');
  });

  it("allows reading hook-written file", () => {
    expect(readFileSync(out, "utf8")).toContain('"ready":true');
  });
});

it("flags parent traversal escaping a fixture directory", () => {
  expect(readFileSync("fixtures/../.github/workflows/ci.yml", "utf8")).toContain(
    "persist-credentials: false",
  );
  expect(readFileSync(join(import.meta.dirname, "fixtures", "..", "package.json"), "utf8")).toMatch(
    /"name":/,
  );
});
