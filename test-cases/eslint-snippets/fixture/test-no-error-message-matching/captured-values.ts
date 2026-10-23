import { expect, test } from "vitest";
declare const error: Error;
const text = error.message;
expect(text).toBe("missing");
const bracket = error["message"];
expect(bracket).not.toContain("missing");
const template = error[`message`];
const forwarded = (template as string)!;
expect(forwarded).toMatch["call"](null, "missing");
const optional = error?.message;
expect(optional).toEqual("missing");
test("captured checks", () => {
  const local = (0, text);
  local.includes("missing");
  local === "missing";
  "missing" !== local;
});
