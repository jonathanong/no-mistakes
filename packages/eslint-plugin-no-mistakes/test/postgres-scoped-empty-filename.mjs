import { createRequire } from "node:module";
import { chdir } from "node:process";

const require = createRequire(import.meta.url);
const parser = require("@typescript-eslint/parser");
const { collectScopedExecutors } = require("../src/rules/postgres-scoped-executors.js");

chdir(process.argv[2]);
const source = [
  'import { openTransaction } from "./transaction";',
  "export async function run() {",
  "  const tx = await openTransaction();",
  "  await tx('BEGIN');",
  "}",
].join("\n");
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
