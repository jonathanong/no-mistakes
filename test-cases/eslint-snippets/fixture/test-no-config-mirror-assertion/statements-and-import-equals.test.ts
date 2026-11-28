import fs = require("node:fs");
import { expect, it } from "vitest";

function loadCi() {
  return fs.readFileSync(".github/workflows/ci.yml", "utf8");
}

export function loadPackage() {
  return fs.readFileSync("package.json", "utf8");
}

function loadTry() {
  try {
    return fs.readFileSync(".github/workflows/ci.yml", "utf8");
  } catch {
    return "";
  }
}

if (fs.readFileSync(".nvmrc", "utf8").includes("22")) {
  void loadTry();
}

it("flags import-equals readFileSync and function-return read helpers without crashing", () => {
  expect(loadCi()).toContain("persist-credentials: false");
  expect(loadPackage()).toContain("private");
  const raw = fs.readFileSync(".github/workflows/ci.yml", "utf8");
  expect(raw).toContain("actions/checkout");
});
