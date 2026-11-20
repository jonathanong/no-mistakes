import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "vitest";

execFileSync("pnpm", ["exec", "widget-audit", "check"]);
execFileSync("npx", ["widget-audit", "check"]);
execFileSync("pnpm", ["dlx", "widget-audit", "check"]);
execFileSync("widget-audit", ["check"]);

const widgetAudit = "./node_modules/.bin/widget-audit";
execFileSync(widgetAudit, ["check"]);

execFileSync("node", ["fixtures/print-result.js"]);
execFileSync("not-widget-audit", ["check"]);

const root = mkdtempSync(join(tmpdir(), "widgets-"));
execFileSync("widget-audit", ["check"], { cwd: root });
execFileSync("widget-audit", ["check"], { cwd: root, "order.id": "1" });
spawnSync("widget-audit", ["check"], { cwd: process.cwd() });
execFileSync("widget-audit", ["check"], { encoding: "utf8" });

test("unbounded repository subprocess", () => {
  execFileSync("pnpm", ["exec", "widget-audit", "check", "--json"]);
});

test("unresolved working directory", () => {
  const options: object = JSON.parse("{}");
  execFileSync("widget-audit", ["check"], options);
});
