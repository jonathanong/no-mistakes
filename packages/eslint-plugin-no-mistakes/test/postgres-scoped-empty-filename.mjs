import { createRequire } from "node:module";
import { chdir } from "node:process";
import { fixture } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const parser = require("@typescript-eslint/parser");
const { collectScopedExecutors } = require("../src/rules/postgres-scoped-executors.js");

// argv[3] stays empty so packageRoot cannot treat this process as the package.
// The program text lives in the saved fixture, not in this helper.
chdir(process.argv[2]);
const source = fixture("postgres-scoped/empty-filename/orders.ts");
const program = parser.parse(source, {
  ecmaVersion: 2024,
  sourceType: "module",
  range: true,
});
const found = collectScopedExecutors(
  program,
  {
    importSpecifier: "@example/db",
    executorFactoryNames: ["openTransaction"],
    executorTypeNames: ["TxExecutor"],
  },
  process.argv[3],
);
process.stdout.write(String(found.length));
