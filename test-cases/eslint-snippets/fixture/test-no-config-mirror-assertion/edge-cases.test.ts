import { execFileSync, execSync, spawnSync } from "node:child_process";
import type { Dirent } from "node:fs";
import {
  copyFileSync,
  default as fsDefault,
  mkdtempDisposableSync,
  readFile,
  readFileSync,
  type Stats,
} from "node:fs";
import { mkdtemp, readFile as readFileAsync } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const baseDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(join(baseDir, "sub"));
const subDir = join("workflows", "ci.yml");
const loadPkg = () => JSON.parse(readFileSync("package.json", "utf8"));
const loadCiAsync = async () => await readFileAsync(".github/workflows/ci.yml", "utf8");
const readAlias = fsDefault.readFileSync;
const { ["readFileSync"]: computedRead } = { readFileSync };
const BASE_ARR = ["a"];
const BASE_OBJ = { ok: true };
const UNDEF = undefined;
const NUM = -1;
const FLAG = "ci";

function multiParam(a: string, b: string) {
  return readFileSync(join(a, b), "utf8");
}

function nonReturnHelper(p: string) {
  const x = readFileSync(p, "utf8");
  return x;
}

function callbackHelper(p: string) {
  return readFile(p, () => {});
}

it("covers dirname, computed const segments, helpers, and literal shapes", async () => {
  const ciPath = join(repoRoot, ".github", subDir);
  const text = readAlias(ciPath, "utf8");
  expect(text ?? "").toContain(`persist-${FLAG}`);
  expect(text || "").toContain("jobs:");
  expect(String(readFileSync(".nvmrc"))).toBe("22");
  expect(loadPkg().count).toBe(NUM);
  expect(loadPkg().missing).toBe(UNDEF);
  expect(loadPkg().voided).toBe(void 0);
  expect(loadPkg().items[0]).toBe("a");
  expect(loadPkg().items).toEqual([...BASE_ARR, , "b"]);
  expect(loadPkg().meta).toEqual({ ...BASE_OBJ, ["key"]: 1 });
  expect(await loadCiAsync()).toContain("jobs:");

  const [first, ...rest] = loadPkg().items;
  const { name = "fallback", ...other } = loadPkg();
  expect(first).toBe("a");
  expect(rest).toEqual(["b"]);
  expect(name).toBe("demo");
  expect(other).toEqual({});
});

it("allows dynamic, temp disposable, copyFile, execSync, and mixed let cases", async () => {
  const disp = mkdtempDisposableSync(join(tmpdir(), "d-"));
  expect(readFileSync(join(disp.path, "a.json"), "utf8")).toContain("a");
  const asyncTmp = await mkdtemp(join(tmpdir(), "a-"));
  expect(readFileSync(join(dirname(asyncTmp), "a.json"), "utf8")).toContain("a");

  const dynArgs = ["a", "b"];
  expect(readFileSync(join(...dynArgs), "utf8")).toContain("a");
  expect(readFileSync(join(repoRoot, "fixtures/a.yml"), "utf8")).toContain("a");
  expect(readFileSync(join(repoRoot, computedRead("x")), "utf8")).toContain("a");
  expect(readFileSync(dirname(computedRead("x")), "utf8")).toContain("a");
  expect(readFileSync(new URL(computedRead("x"), import.meta.url), "utf8")).toContain("a");

  const copied = join(repoRoot, "copied.json");
  copyFileSync("package.json", copied);
  expect(readFileSync(copied, "utf8")).toContain("demo");

  const shPath = join(repoRoot, "scripts/shell.sh");
  execSync(`bash ${shPath}`);
  execSync(shPath);
  expect(readFileSync(shPath, "utf8")).toContain("set -e");

  const argv = [join(repoRoot, "scripts/arr.sh")];
  spawnSync("bash", argv);
  spawnSync("bash", [...argv]);
  execFileSync(shPath);
  expect(readFileSync(join(repoRoot, "scripts/arr.sh"), "utf8")).toContain("set -e");

  let mixed = "initial";
  mixed = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(mixed).toContain("jobs:");

  let reassigned = "";
  reassigned = readFileSync(".github/workflows/ci.yml", "utf8");
  reassigned = "other";
  expect(reassigned).toContain("jobs:");

  const pkg = JSON.parse(readFileSync("package.json", "utf8"));
  const dynamicKey = Math.random() > 0.5 ? "a" : "b";
  expect(pkg[dynamicKey]).toBe("x");
  expect(pkg).toHaveLength(1);
  expect(pkg.items).toEqual(...dynArgs);
  expect(readFileSync(".github/workflows/ci.yml").toString()).toContain("jobs:");
  readFile(".github/workflows/ci.yml", () => {});
  void readFileAsync(".github/workflows/ci.yml", "utf8").then(() => {});
  void multiParam("a", "b");
  void nonReturnHelper("a");
  void callbackHelper(".github/workflows/ci.yml");
  const stat: Stats | null = null;
  const dirent: Dirent | null = null;
  void stat;
  void dirent;
});

function outsideTest(p: string) {
  const t = readFileSync(p, "utf8");
  return t;
}
void outsideTest("package.json");
expect(readFileSync(".nvmrc", "utf8")).toContain("22");
