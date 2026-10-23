import { expect, test } from "vitest";
declare let error: { message: string; code: string };
// Capture the stable contract at the same point as the original message read.
const code = error.code;
expect(code).toBe("MISSING");
const bracket = error["code"];
expect(bracket).not.toBe("MISSING");
const template = error[`code`];
const forwarded = (template as string)!;
expect(forwarded).toBe["call"](null, "MISSING");
const optional = error?.code;
expect(optional).toEqual("MISSING");
test("stable captured checks", () => {
  const local = (0, code);
  // Replace the substring condition with a stable code comparison.
  local === "MISSING";
  local === "MISSING";
  "MISSING" !== local;
});

declare function loadError(): { message: string; code: string };
declare function display(message: string): void;
// Retain another message consumer without evaluating the original receiver twice.
const failure = loadError();
const displayed = failure.message;
const stable = failure.code;
display(displayed);
expect(stable).toBe("MISSING");
