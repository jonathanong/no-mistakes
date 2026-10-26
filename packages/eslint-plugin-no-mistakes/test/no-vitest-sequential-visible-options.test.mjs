import { describe, expect, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

describe("visible sequential options in opaque forwarding", () => {
  it("preserves visible options while leaving unknown argument arrays alone", () => {
    const findings = lint(
      fixture("no-vitest-sequential/visible-forwarded-options.test.ts"),
      { "no-mistakes/no-vitest-sequential": "error" },
      "fixture.test.ts",
    );
    expect(findings.map(({ line, messageId }) => [line, messageId])).toEqual(
      [4, 5, 6].map((line) => [line, "sequential"]),
    );
  });
});
