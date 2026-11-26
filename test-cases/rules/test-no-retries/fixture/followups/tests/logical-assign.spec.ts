import { test as base } from "@playwright/test";

let suite: typeof base | undefined;
suite ??= base;

suite.describe.serial("ordered", () => {});
suite.describe.configure({ retries: 2 });

let selfRef: typeof base | undefined = base;
selfRef = selfRef;
selfRef.describe.serial("self assign", () => {});

const make = () => (process.env.CI ? base : ({} as typeof base));
make().describe.serial("arrow expr", () => {});

const pick = () => base;
(process.env.CI ? pick() : base).describe.serial("expr fn", () => {});

const runExpr = function (s: typeof base) {
  s.describe.serial("fn expr", () => {});
};
runExpr(base);
