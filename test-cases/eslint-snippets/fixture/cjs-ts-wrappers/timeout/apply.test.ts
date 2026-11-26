import { test } from "vitest";
test.apply(null, ["slow", (ctx) => { ctx.onTestFinished(() => {}, 60000); }] as const);
