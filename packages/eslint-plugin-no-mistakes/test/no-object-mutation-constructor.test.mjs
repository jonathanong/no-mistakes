import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, messages } from "./helpers.mjs";

const rule = "no-object-mutation";
const names = ["constructor", "constructor-controls", "bare-this", "bare-this-controls"];
const cases = names.map((name) => {
  const filename = `no-object-mutation-${name}.ts`;
  return { code: fixture(filename), filename, options: [{ allowThis: false }] };
});

describe("no-object-mutation constructor initialization and bare this calls", () => {
  it("allows direct initialization of own declared fields", () => {
    expect(messages(cases[0].code, rule, { allowThis: false }, cases[0].filename)).toEqual([]);
  });
  it("abstains from unknown bare this mutators", () => {
    expect(messages(cases[2].code, rule, { allowThis: false }, cases[2].filename)).toEqual([]);
  });
  it("preserves other constructor writes and known mutators", () => {
    for (const index of [1, 3]) {
      expect(
        messages(cases[index].code, rule, { allowThis: false }, cases[index].filename).length,
      ).toBe(index === 1 ? 17 : 12);
    }
  });
  it("snapshots every saved shape", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite(
          rule,
          { ecmaVersion: 2024, sourceType: "module", parser: tsParser },
          { valid: [cases[0], cases[2]], invalid: [cases[1], cases[3]] },
        ),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-constructor.snap");
  });
});
