import { readFileSync } from "node:fs";
import yaml from "yaml";
import { expect, it } from "vitest";

it("allows comparing two tracked files directly", () => {
  const ci = readFileSync(".github/workflows/ci.yml", "utf8");
  const pkg = JSON.parse(readFileSync("package.json", "utf8"));
  expect(ci).toContain(pkg.engines.node);
});

it("allows comparing parsed fields across two files", () => {
  const rootPkg = JSON.parse(readFileSync("package.json", "utf8"));
  const subPkg = JSON.parse(readFileSync("packages/app/package.json", "utf8"));
  expect(subPkg.version).toBe(rootPkg.version);
});

it("allows per-item assertions over a set enumerated from a second file", () => {
  const ci = readFileSync(".github/workflows/ci.yml", "utf8");
  const workspace = yaml.parse(readFileSync("pnpm-workspace.yaml", "utf8"));
  for (const pkg of workspace.packages) {
    expect(ci).toContain(pkg);
  }
});

it("allows forEach cross-file invariant alongside a sanity literal check", () => {
  const readme = readFileSync("README.md", "utf8");
  const pkg = JSON.parse(readFileSync("package.json", "utf8"));
  expect(readme).toContain("## Scripts");
  Object.keys(pkg.scripts).forEach((script) => {
    expect(readme).toContain(script);
  });
});

it("allows regex constructed from another file", () => {
  const ci = readFileSync(".github/workflows/ci.yml", "utf8");
  const version = readFileSync(".nvmrc", "utf8").trim();
  expect(ci).toMatch(new RegExp(`node-version:\\s*${version}`));
});
