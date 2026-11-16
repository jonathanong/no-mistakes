import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, messages } from "./helpers.mjs";

const rule = "no-object-mutation";
const typed = "no-object-mutation-receiver.ts";
const jsdoc = "no-object-mutation-receiver.js";
const controls = "no-object-mutation-receiver-controls.ts";
const jsControls = "no-object-mutation-receiver-controls.js";
const languageOptions = { ecmaVersion: 2024, sourceType: "module", parser: tsParser };
const savedCase = (name, options) => ({ code: fixture(name), filename: name, options });
const mutations = (count) => Array.from({ length: count }, () => "mutation");

describe("no-object-mutation declared receivers", () => {
  it("allows TypeScript this parameters and their lexical arrows", () => {
    expect(messages(fixture(typed), rule, undefined, typed)).toEqual(mutations(1));
  });

  it("allows JSDoc this declarations and their lexical arrows", () => {
    expect(messages(fixture(jsdoc), rule, undefined, jsdoc)).toEqual([]);
  });

  it("keeps undeclared methods and nested regular functions reported", () => {
    expect(messages(fixture(controls), rule, undefined, controls)).toEqual(mutations(6));
  });

  it("requires attached JSDoc and preserves ordinary function boundaries", () => {
    expect(messages(fixture(jsControls), rule, undefined, jsControls)).toEqual(mutations(11));
  });

  it("honors allowThis false for both receiver declarations", () => {
    expect(messages(fixture(typed), rule, { allowThis: false }, typed)).toEqual(mutations(12));
    expect(messages(fixture(jsdoc), rule, { allowThis: false }, jsdoc)).toEqual(mutations(6));
  });

  it("snapshots declared receivers, undeclared functions, and disabled allowance", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite(rule, languageOptions, {
          valid: [savedCase(jsdoc)],
          invalid: [
            savedCase(typed),
            savedCase(controls),
            savedCase(jsControls),
            savedCase(typed, [{ allowThis: false }]),
            savedCase(jsdoc, [{ allowThis: false }]),
          ],
        }),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-receiver.snap");
  });
});
