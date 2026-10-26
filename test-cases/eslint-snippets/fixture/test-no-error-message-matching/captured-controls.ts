import { expect, test } from "vitest";
declare const error: Error;
declare const response: { body: { message: string } };
declare const key: string;
const body = response.body.message;
expect(body).toBe("allowed");
const dynamic = error[key];
expect(dynamic).toBe("unknown");
let mutable = error.message;
mutable = "replacement";
expect(mutable).toBe("replacement");
var legacy = error.message;
expect(legacy).toBe("unknown");
const { message: destructured } = error;
expect(destructured).toBe("unknown");
const readMessage = () => error.message;
const returned = readMessage();
expect(returned).toBe("unknown");
const holder = { text: error.message };
const stored = holder.text;
expect(stored).toBe("unknown");
const text = error.message;
function shadowed(text: string) {
  expect(text).toBe("unrelated");
}
function localShadow() {
  const text = "unrelated";
  expect(text).toBe("unrelated");
}
// String operations and comparisons retain the rule's existing test-only scope.
text.includes("outside test");
text === "outside test";
test("controls", () => {
  text.startsWith("outside the supported methods");
  text > "outside the supported operators";
  text === other;
});
expect().toBe("no captured value");
// Erased wrappers do not make mutable or exempt body captures eligible.
let checked = error.message satisfies string;
expect(checked).toBe("unknown");
const checkedBody = response.body.message satisfies string;
expect(checkedBody).toBe("allowed");
