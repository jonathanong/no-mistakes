const fs = require("node:fs");
const { ["readFileSync"]: readSync } = require("node:fs");
const { readFile } = require("node:fs/promises");
const { resolve } = require("node:path");
const { expect, it } = require("vitest");

it("flags CommonJS readFileSync", () => {
  const text = fs.readFileSync(resolve(__dirname, "../.github/workflows/ci.yml"), "utf8");
  expect(text).toContain("persist-credentials: false");
  const pkg = readSync("package.json", "utf8");
  expect(pkg).toContain("private");
});

it("flags CommonJS fs/promises readFile", async () => {
  const text = await readFile("package.json", "utf8");
  expect(text).toMatch(/"private":\s*true/);
});
