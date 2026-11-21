import {
  exec,
  execFileSync,
  execSync,
  fork,
  spawn,
  spawnSync,
} from "node:child_process";
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

exec("pnpm exec widget-audit check");
execSync("pnpm exec widget-audit check");
execSync("pnpm exec widget-audit check", { encoding: "utf8" });
const widgetCommand = "pnpm exec widget-audit check";
execSync(widgetCommand);
execSync("pnpm exec other-audit check");
execSync("pnpm exec widget-audit list");
execSync("cd widgets && widget-audit check");
execSync("cd /repo && widget-audit check", { cwd: root });
execSync("cd fixtures && widget-audit check", { cwd: process.cwd() });
execSync("cd ../widgets && widget-audit check");
execSync("cd fixtures/$DIR && widget-audit check");
execSync("cd fixtures/`item` && widget-audit check");
execSync("cd 'fixtures/$item' && widget-audit check");
execSync("cd . && widget-audit check");
execSync("cd / && widget-audit check");
execSync("cd .. && widget-audit check");
execSync("cd . && widget-audit check", { cwd: root });
execSync("cd / && widget-audit check", { cwd: root });
execSync("cd .. && widget-audit check", { cwd: root });
execFileSync("pnpm", ["-C", "/repo", "exec", "widget-audit", "check"], { cwd: root });
execFileSync("pnpm", ["--dir", "/repo", "exec", "widget-audit", "check"], { cwd: root });
execFileSync("yarn", ["--cwd", "/repo", "widget-audit", "check"], { cwd: root });
execFileSync("yarn", ["--cwd=/repo", "widget-audit", "check"], { cwd: root });
execFileSync("pnpm", ["-C", "fixtures", "exec", "widget-audit", "check"], { cwd: root });
execFileSync("pnpm", ["--filter", "widgets", "exec", "widget-audit", "check"], { cwd: root });
execFileSync("npx", ["pnpm", "-C", "/repo", "exec", "widget-audit", "check"], { cwd: root });
execSync('ITEM="placeholder widget-audit check extra"');
execSync("ITEM=1 widget-audit check");
execSync("cd widgets && ITEM=1 pnpm exec widget-audit check");
execSync("cd widgets; widget-audit check");
execSync("cd widgets || widget-audit check");
execSync("true && widget-audit check");
execSync("true&&widget-audit check");
execSync("true || widget-audit check");
execSync("true; widget-audit check");
execSync("\"widget-audit\" check");
execSync("widget-au''dit check");
execSync("cd fixtures && widget-audit check && cd /repo && widget-audit check", { cwd: root });
spawn("pnpm exec widget-audit check");
spawn("pnpm exec widget-audit check", { shell: false });
spawn("pnpm exec widget-audit check", { shell: true });
spawn("pnpm exec widget-audit check", { shell: true, cwd: root });
spawn("pnpm", ["exec", "widget-audit", "check"], { shell: true });
fork("./node_modules/.bin/widget-audit", ["check"]);
fork("./node_modules/.bin/other-audit", ["check"]);
fork("./widget-audit.js", ["check"]);
fork("./widgets/widget-audit.mjs", ["check"]);
fork("./widget-audit.cjs", ["check"]);
fork("./other-audit.js", ["check"]);

execFileSync("pnpm", ["-C", "dir", "exec", "widget-audit", "check"]);
execFileSync("pnpm", ["--filter", "widgets", "exec", "widget-audit", "check"]);
execFileSync("pnpm", ["-r", "exec", "widget-audit", "check"]);
execFileSync("pnpm", ["run", "widget-audit", "check"]);
execFileSync("npx", ["--no-install", "widget-audit", "check"]);
execFileSync("npx", ["-p", "widget-tools", "widget-audit", "check"]);
execFileSync("npm", ["exec", "widget-audit", "check"]);
execFileSync("yarn", ["widget-audit", "check"]);
execFileSync("yarn", ["dlx", "widget-audit", "check"]);
execFileSync("yarn", ["run", "widget-audit", "check"]);
execFileSync("yarn", ["--cwd", "widgets", "widget-audit", "check"]);
execFileSync("pnpm", ["--reporter", "silent", "exec", "widget-audit", "check"]);
execFileSync("pnpm", ["--ignore-scripts", "exec", "widget-audit", "check"]);
execFileSync("bunx", ["widget-audit", "check"]);
execFileSync("npx", ["pnpm", "exec", "widget-audit", "check"]);
execFileSync("npx", ["pnpm", "-C", "dir", "exec", "widget-audit", "check"]);

const parts = ["pnpm", "exec", "widget-audit", "check"];
execSync(parts.join(" "));

test("unbounded repository subprocess", () => {
  execFileSync("pnpm", ["exec", "widget-audit", "check", "--json"]);
});

test("unresolved working directory", () => {
  const options: object = JSON.parse("{}");
  execFileSync("widget-audit", ["check"], options);
});
