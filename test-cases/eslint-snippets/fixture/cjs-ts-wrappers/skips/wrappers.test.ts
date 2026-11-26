import * as v from "vitest";
v.test("context", (ctx) => { ctx.skip(); });
v.test("cast member", (ctx) => { (ctx.skip as any)(); });
v.test("destructured", ({ skip }) => { (skip as any)(); });
v.test("cast callback", ((ctx: any) => { ctx.skip(); }) as any);
(v as typeof v).test.skip("namespace cast", () => {});
