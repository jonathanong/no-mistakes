import { test } from "vitest";

const key = "sequential";
const enabled = true;
test[key as string]("dynamic member", () => {});
test("dynamic option key", { [key as string]: true }, () => {});
test("dynamic option value", { sequential: enabled as boolean }, () => {});
test("false", { sequential: false as const }, () => {});
test("ordinary option", { ["concurrent" as const]: true }, () => {});
(test as typeof test)("ordinary runner", () => {});
function helper(test: any) {
  (test.sequential as any)("shadowed parameter", () => {});
}
const other = { sequential() {} };
(other.sequential as Function)("unrelated receiver", () => {});
// no-mistakes-disable-next-line no-vitest-sequential -- intentional suppression probe
(test.sequential as typeof test)("suppressed next line", () => {});
(test.sequential as typeof test)("suppressed line", () => {}); // no-mistakes-disable-line no-vitest-sequential
