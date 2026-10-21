import { read } from "@example/db";
import stringTable from "./string-table.mts";
import templateTable from "./template-table.mts";
import identTable from "./ident-table.mts";
import parenTable from "./paren-table.mts";
import assertedTable from "./as-const-table.mts";
import parenIdent from "./paren-ident-table.mts";
import substituted from "./substituted.mts";

// A bare `export default NAME` follows that const. A string, a no-substitution
// template, parentheses, and `as const` are expression defaults. A template
// with a substitution stays unanalyzable.
export function fromString() {
  return read(`SELECT id FROM ${stringTable} WHERE true`);
}

export function fromTemplate() {
  return read(`SELECT id FROM ${templateTable} WHERE true`);
}

export function fromIdent() {
  return read(`SELECT id FROM ${identTable} WHERE true`);
}

export function fromParen() {
  return read(`SELECT id FROM ${parenTable} WHERE true`);
}

export function fromAsserted() {
  return read(`SELECT id FROM ${assertedTable} WHERE true`);
}

export function fromParenIdent() {
  return read(`SELECT id FROM ${parenIdent} WHERE true`);
}

export function fromSubstituted(accountId: string) {
  return read(`SELECT id FROM ${substituted} WHERE account_id = $1`, [accountId]);
}
