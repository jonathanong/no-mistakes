import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, messages } from "./helpers.mjs";

const rule = "no-object-mutation";
const cases = ["private-constructor", "private-constructor-controls"].map((name) => {
  const filename = `no-object-mutation-${name}.ts`;
  return { code: fixture(filename), filename, options: [{ allowThis: false }] };
});

describe("no-object-mutation private constructor fields", () => {
  it("allows direct initialization of own private fields", () => {
    expect(messages(cases[0].code, rule, { allowThis: false }, cases[0].filename)).toEqual([]);
  });
  it("keeps public names, static fields, compound writes, callbacks and methods reportable", () => {
    expect(messages(cases[1].code, rule, { allowThis: false }, cases[1].filename)).toEqual(
      Array(5).fill("mutation"),
    );
  });
  it("snapshots private initialization and controls", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite(
          rule,
          { ecmaVersion: 2024, sourceType: "module", parser: tsParser },
          { valid: [cases[0]], invalid: [cases[1]] },
        ),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-private-constructor.snap");
  });
});
