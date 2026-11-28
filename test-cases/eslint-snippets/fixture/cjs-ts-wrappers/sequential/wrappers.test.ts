import * as v from "vitest";
v.test["apply" as const](null, ["batch", { sequential: true }, () => {}]);
const { test: run1 } = v as typeof v;
run1.sequential("cast namespace", () => {});
const { ["test" as const]: run2 } = v;
run2.sequential("computed key", () => {});
const { test: run3 } = require("vitest") as typeof import("vitest");
run3.sequential("require cast", () => {});
