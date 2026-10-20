import { lint, messages } from "./helpers.mjs";

export const RULE = "postgres-no-async-sql-builder";
export const NAMES = { builders: [{ names: ["build"] }] };
export const SQL = { builders: [{ callees: ["sql"], source: "@example/db" }] };

// Existing callers named `read` stand in for a same-file consumer whose
// parameter is a string, not a Promise. Unresolved `read` is tested separately.
export function withSyncRead(code) {
  return `${code}\nfunction read(query: string): string { return query; }\n`;
}

export function ids(code, option = NAMES) {
  return messages(withSyncRead(code), RULE, option, "app.ts");
}

export function reported(code, option = NAMES) {
  return lint(withSyncRead(code), { [`no-mistakes/${RULE}`]: ["error", option] }, "app.ts");
}
