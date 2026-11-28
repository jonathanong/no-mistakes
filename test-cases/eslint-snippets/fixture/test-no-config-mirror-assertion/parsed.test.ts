import { readFileSync } from "node:fs";
import jsYaml from "js-yaml";
import yaml, { parse as parseYaml } from "yaml";
import { expect, it } from "vitest";

it("flags JSON.parse with destructuring and Object.keys", () => {
  const raw = readFileSync("package.json", "utf8");
  const { name, scripts } = JSON.parse(raw);
  expect(name).toBe("demo");
  expect(Object.keys(scripts)).toContain("build");
});

it("flags yaml.parse and renamed yaml parse", () => {
  const ci = yaml.parse(readFileSync(".github/workflows/ci.yml", "utf8"));
  expect(ci.jobs.test["timeout-minutes"]).toBe(15);

  const doc = parseYaml(readFileSync(".github/workflows/release.yml", "utf8"));
  expect(doc.on).toEqual(["push"]);
});

it("flags yaml.parseDocument().toJSON() and js-yaml load", () => {
  const parsed = yaml.parseDocument(readFileSync(".github/workflows/ci.yml", "utf8")).toJSON();
  expect(parsed.permissions).toStrictEqual({ contents: "read" });

  const loaded = jsYaml.load(readFileSync("pnpm-workspace.yaml", "utf8"));
  expect(loaded.packages).toEqual(["packages/*"]);
});

it("flags Buffer toString and trim", () => {
  const version = readFileSync(".nvmrc").toString("utf8").trim();
  expect(version).toBe("22");
});
