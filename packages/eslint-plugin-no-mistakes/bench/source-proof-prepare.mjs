import assert from "node:assert/strict";
import {
  rmSync,
  cpSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
const here = dirname(fileURLToPath(import.meta.url));
export const repository = resolve(here, "../../..");
export const packageRoot = resolve(here, "..");
export const fixture = resolve(
  repository,
  "test-cases/eslint-plugin/async-call-disposition/fixture/source-proof-performance",
);
const require = createRequire(resolve(packageRoot, "package.json"));
export const oxlint = resolve(repository, "node_modules/oxlint/bin/oxlint");
export function hashSources(source) {
  const hash = createHash("sha256");
  function visit(path, relative = "") {
    for (const entry of readdirSync(path, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name),
    )) {
      const name = `${relative}/${entry.name}`;
      if (entry.isDirectory()) visit(resolve(path, entry.name), name);
      else hash.update(name).update(readFileSync(resolve(path, entry.name)));
    }
  }
  visit(source);
  return hash.digest("hex");
}
export function archiveSources(revision, destination) {
  rmSync(destination, { recursive: true, force: true });
  mkdirSync(destination, { recursive: true });
  const archive = execFileSync(
    "git",
    ["archive", revision, "packages/eslint-plugin-no-mistakes/src"],
    { cwd: repository, maxBuffer: 16 * 1024 * 1024 },
  );
  execFileSync("tar", ["-x", "--strip-components=3", "-C", destination], { input: archive });
}
export function prepare(root, callers = 4096, variant = "dense", baseline = "HEAD") {
  assert.ok(Number.isInteger(callers) && callers > 0);
  assert.ok(["dense", "sparse"].includes(variant));
  mkdirSync(root, { recursive: true });
  cpSync(fixture, resolve(root, "workload"), {
    recursive: true,
    filter: (path) =>
      ![resolve(fixture, "callers"), resolve(fixture, "dense")].some(
        (excluded) => path === excluded || path.startsWith(`${excluded}/`),
      ),
  });
  mkdirSync(resolve(root, "workload/callers"), { recursive: true });
  const caller = resolve(fixture, variant === "dense" ? "dense/caller.ts" : "callers/caller.ts");
  cpSync(caller, resolve(root, "workload/callers/caller.ts"));
  for (let index = 1; index < callers; index++) {
    cpSync(caller, resolve(root, `workload/callers/caller-${String(index).padStart(5, "0")}.ts`));
  }
  for (const name of ["typescript", "@typescript-eslint/parser"]) {
    const target = resolve(root, "peers/node_modules", name);
    mkdirSync(dirname(target), { recursive: true });
    symlinkSync(dirname(require.resolve(`${name}/package.json`)), target, "junction");
  }
  const manifest = resolve(packageRoot, "package.json");
  for (const name of ["peers/before", "peers/after", "absent/plugin"]) {
    mkdirSync(resolve(root, name), { recursive: true });
    archiveSources(baseline, resolve(root, name, "src"));
    cpSync(manifest, resolve(root, name, "package.json"));
  }
  const absent = createRequire(resolve(root, "absent/plugin/package.json"));
  for (const name of ["typescript", "@typescript-eslint/parser"]) {
    try {
      absent.resolve(name);
      throw new Error(`Absent peer resolved: ${name}`);
    } catch (error) {
      if (error.code !== "MODULE_NOT_FOUND") throw error;
    }
  }
  const checkerRequire = createRequire(require.resolve("oxlint-tsgolint/package.json"));
  const checker = checkerRequire.resolve(
    `@oxlint-tsgolint/${process.platform}-${process.arch}/tsgolint${process.platform === "win32" ? ".exe" : ""}`,
  );
  const metadata = {
    baseline: execFileSync("git", ["rev-parse", baseline], {
      cwd: repository,
      encoding: "utf8",
    }).trim(),
    sourceHash: hashSources(resolve(root, "peers/before/src")),
    fixtureHash: hashSources(resolve(root, "workload")),
    variant,
    callerLines: readFileSync(caller, "utf8").split("\n").length - 1,
    callerBytes: readFileSync(caller).length,
    jsxFiles: 0,
    callers,
    threads: 4,
    node: process.version,
    oxlint: require("oxlint/package.json").version,
    typescript: require("typescript/package.json").version,
    parser: require("@typescript-eslint/parser/package.json").version,
    checker,
  };
  writeFileSync(resolve(root, "metadata.json"), JSON.stringify(metadata, null, 2));
  return metadata;
}
