import { test } from "vitest";

// Type-only wrappers must not hide serial execution.
(test.sequential as typeof test)("as callee", () => {});
(test.sequential satisfies typeof test)("satisfies callee", () => {});
test.sequential!("non-null callee", () => {});
(<typeof test>test.sequential)("assertion callee", () => {});
(test.sequential<{}>)("instantiated callee", () => {});
(test as typeof test).sequential("wrapped runner", () => {});
test["sequential" as const]("wrapped string key", () => {});
test[`sequential` satisfies string]("wrapped template key", () => {});
((0, test.sequential as typeof test) satisfies typeof test)("nested sequence", () => {});
const serial = test.sequential as typeof test;
serial("wrapped alias", () => {});
test("wrapped options", ({ sequential: true } satisfies object), () => {});
test("wrapped true", { sequential: true as const }, () => {});
test("wrapped option key", { ["sequential" as const]: true }, () => {});
test("wrapped option template", { [`sequential` satisfies string]: true }, () => {});
(test.sequential as typeof test).call(null, "wrapped forwarding", () => {});
// A wrapped parameterized test still reports once, at the builder call.
(test.sequential.each([1]) as typeof test)("wrapped each", () => {});
