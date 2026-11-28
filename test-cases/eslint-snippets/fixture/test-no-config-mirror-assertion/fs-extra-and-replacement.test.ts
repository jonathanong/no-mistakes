import { readFile } from "fs-extra";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("flags fs-extra callback readFile", () => {
  readFile("package.json", "utf8", (_err, text) => {
    expect(text).toContain("name");
  });
});

it("flags fs-extra promise readFile", async () => {
  const text = await readFile("package.json", "utf8");
  expect(text).toContain("name");
});

it("ignores a read value replaced by an empty literal", () => {
  let text = readFileSync("package.json", "utf8");
  text = "";
  expect(text).toBe("");
});

it("flags an empty initializer followed by a file read", () => {
  let text = "";
  text = readFileSync("package.json", "utf8");
  expect(text).toContain("name");
});

function* load(path: string) {
  return readFileSync(path, "utf8");
}

it("ignores a generator helper that reads nothing", () => {
  expect(load("package.json").toString()).toBe("[object Generator]");
});
