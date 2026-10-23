import { expect, test } from "vitest";
declare let error: { message: string; code: string };
// Capture the stable contract at the same point as the original message read.
const code = error.code;
expect(code).toBe("MISSING");
const bracket = error["code"];
expect(bracket).not.toContain("MISSING");
const template = error[`code`];
const forwarded = (template as string)!;
expect(forwarded).toMatch["call"](null, "MISSING");
const optional = error?.code;
expect(optional).toEqual("MISSING");
test("stable captured checks", () => {
  const local = (0, code);
  local.includes("MISSING");
  local === "MISSING";
  "MISSING" !== local;
});
