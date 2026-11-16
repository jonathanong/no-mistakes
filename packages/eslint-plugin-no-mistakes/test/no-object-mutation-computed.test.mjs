import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, messages } from "./helpers.mjs";

const rule = "no-object-mutation";
const local = "no-object-mutation-computed-local.ts";
const shared = "no-object-mutation-computed-shared.ts";
const languageOptions = { ecmaVersion: 2024, sourceType: "module", parser: tsParser };
const savedCase = (name) => ({ code: fixture(name), filename: name });

describe("no-object-mutation computed receivers", () => {
  it("keeps all four local builder shapes owned", () => {
    expect(messages(fixture(local), rule, undefined, local)).toEqual([]);
  });

  it("retains shared bases, shared children, and escaped child identities", () => {
    expect(messages(fixture(shared), rule, undefined, shared)).toEqual(
      Array.from({ length: 8 }, () => "mutation"),
    );
  });

  it("snapshots local builders and shared receiver diagnostics", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite(rule, languageOptions, {
          valid: [savedCase(local)],
          invalid: [savedCase(shared)],
        }),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-computed.snap");
  });
});
