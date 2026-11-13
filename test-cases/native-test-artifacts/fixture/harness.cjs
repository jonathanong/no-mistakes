#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const args = process.argv.slice(2);
const suite = path.basename(process.argv[1]).replace(/-fixture$/, "");
const filter = args[0]?.startsWith("--") ? undefined : args[0];
const tests = JSON.parse(fs.readFileSync(process.env.NATIVE_FIXTURE_CASES))[suite];
const selected = tests.filter((name) => !filter || (args.includes("--exact") ? name === filter : name.includes(filter)));
if (args.includes("--list")) {
  if (process.env.NATIVE_FIXTURE_LISTING) process.stdout.write(fs.readFileSync(process.env.NATIVE_FIXTURE_LISTING));
  else for (const name of selected) console.log(`${name}: test`);
} else {
  if (!args.includes("--exact") || selected.length !== 1) throw new Error("expected exactly one selected test");
  fs.appendFileSync(process.env.NATIVE_FIXTURE_EXECUTIONS, `${JSON.stringify({ executable: process.argv[1], args, tests: selected, cwd: process.cwd() })}\n`);
}
process.exit(Number(process.env.NATIVE_TEST_FIXTURE_EXIT || 0));
