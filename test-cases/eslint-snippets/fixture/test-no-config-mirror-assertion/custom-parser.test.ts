import { readFileSync } from "node:fs";
import parseIni from "custom-ini";
import { parse as parseToml } from "smol-toml";
import { expect, it } from "vitest";

it("uses configured named and default parsers", () => {
  const cargo = parseToml(readFileSync("Cargo.toml", "utf8"));
  expect(cargo.package.edition).toBe("2024");

  const ini = parseIni(readFileSync(".editorconfig", "utf8"));
  expect(ini.root).toBe(true);
});
