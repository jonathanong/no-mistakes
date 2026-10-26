import { test } from "vitest";
declare const callbacks: unknown[];
// A later spread cannot erase the already visible sequential option.
test.call(null, "spread", { sequential: true }, ...callbacks);
test["call"](null, "static key", { sequential: true }, ...callbacks);
test?.call?.(null, "optional", { sequential: true }, ...callbacks);
// Unknown argument arrays do not expose nested options.
test.apply(null, callbacks);
test.apply(null, ["unknown", ...callbacks]);
test.call(null, "false", { sequential: false }, ...callbacks);
// no-mistakes-disable-next-line no-vitest-sequential
test.call(null, "suppressed", { sequential: true }, ...callbacks);
