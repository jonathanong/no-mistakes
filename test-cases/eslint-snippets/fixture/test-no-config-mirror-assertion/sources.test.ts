import fs, { readFile as readFileCb, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { beforeAll, expect, it } from "vitest";

const readFileAsync = promisify(readFileCb);
const repoRoot = resolve(import.meta.dirname, "..");

function readRepoFile(relPath: string) {
  return readFileSync(join(repoRoot, relPath), "utf8");
}

it("flags readFileSync with literal path", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("persist-credentials: false");
});

it("flags fs.readFileSync with path.join(process.cwd(), ...)", () => {
  const text = fs.readFileSync(join(process.cwd(), ".github/workflows/ci.yml"), "utf8");
  expect(text).toContain("actions/checkout");
});

it("flags fs/promises readFile", async () => {
  const text = await readFile("package.json", "utf8");
  expect(text).toContain('"private": true');
});

it("flags fs.promises.readFile with new URL", async () => {
  const text = await fs.promises.readFile(new URL("../package.json", import.meta.url), "utf8");
  expect(text).toMatch(/"type":\s*"module"/);
});

it("flags resolves matcher on readFile promise", async () => {
  await expect(readFile(".github/workflows/ci.yml", "utf8")).resolves.toContain(
    "persist-credentials: false",
  );
});

it("flags callback readFile", () => {
  readFileCb(fileURLToPath(new URL("../tsconfig.json", import.meta.url)), "utf8", (_err, text) => {
    expect(text).toContain('"strict": true');
  });
});

it("flags promisified readFile", async () => {
  const text = await readFileAsync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("timeout-minutes");
});

it("flags then callback on readFile promise", async () => {
  await readFile(".github/workflows/ci.yml", "utf8").then((text) => {
    expect(text).toContain("permissions:");
  });
});

it("flags same-file helper for repo path and allows fixture path", () => {
  expect(readRepoFile(".github/workflows/ci.yml")).toContain("persist-credentials: false");
  expect(readRepoFile("fixtures/ci.yml")).toContain("persist-credentials: false");
});

let sharedWorkflow = "";
beforeAll(() => {
  sharedWorkflow = readFileSync(".github/workflows/ci.yml", "utf8");
});

it("flags beforeAll let assignment", () => {
  expect(sharedWorkflow).toContain("persist-credentials: false");
});
