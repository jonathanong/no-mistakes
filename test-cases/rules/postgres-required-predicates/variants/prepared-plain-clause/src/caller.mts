import sql from "sql-template-strings";
import { read } from "@example/db";
import { assertImpossible } from "./barrel.mts";
declare function unresolved(value: string): never;
declare function next(): string;

function table(family: string): string {
  switch (family) {
    case "one": return "orders";
    case "two": return "accounts";
    case "three": return "archive_a";
    case "four": return "archive_b";
    case "five": return "archive_c";
    case "six": return "archive_d";
    case "seven": return "archive_e";
    default: return assertImpossible(family as never);
  }
}

// All seven literal table names are inserted into complete ordinary SQL.
export function select(family: string) {
  return read(`SELECT id FROM ${table(family)} WHERE id = $1`, [1]);
}
export function update(family: string) {
  return read(`UPDATE ${table(family)} SET id = $1 WHERE id = $2`, [1, 2]);
}
export function remove(family: string) {
  return read(`DELETE FROM ${table(family)} WHERE id = $1`, [1]);
}

function returningDefault(family: string): string {
  switch (family) {
    case "one": return "orders";
    default: return returnsText(family);
  }
}
function returnsText(value: string): string { return value; }
export function defaultReturns(family: string) {
  return read(`SELECT id FROM ${returningDefault(family)} WHERE id = $1`, [1]);
}

function unresolvedDefault(family: string): string {
  switch (family) {
    case "one": return "orders";
    default: return unresolved(family);
  }
}
export function missingThrow(family: string) {
  return read(`SELECT id FROM ${unresolvedDefault(family)} WHERE id = $1`, [1]);
}

// A call argument with effects cannot be projected as a plain clause.
export function mutableSelector() {
  return read(`SELECT id FROM ${table(next())} WHERE id = $1`, [1]);
}
export function shadowed(table: (family: string) => string, family: string) {
  return read(`SELECT id FROM ${table(family)} WHERE id = $1`, [1]);
}
const alias = table;
export function escaped(family: string) {
  return read(`SELECT id FROM ${alias(family)} WHERE id = $1`, [1]);
}
// Tagged substitutions bind values; they never splice a table identifier.
export function tagged(family: string) {
  return read(sql`SELECT id FROM ${table(family)} WHERE id = ${1}`);
}

function tooMany(family: string): string {
  switch (family) {
    case "a": return "orders";
    case "b": return "archive_b";
    case "c": return "archive_c";
    case "d": return "archive_d";
    case "e": return "archive_e";
    case "f": return "archive_f";
    case "g": return "archive_g";
    case "h": return "archive_h";
    case "i": return "archive_i";
    case "j": return "archive_j";
    case "k": return "archive_k";
    case "l": return "archive_l";
    case "m": return "archive_m";
    case "n": return "archive_n";
    case "o": return "archive_o";
    case "p": return "archive_p";
    case "q": return "archive_q";
    default: return assertImpossible(family as never);
  }
}
export function overBudget(family: string) {
  return read(`SELECT id FROM ${tooMany(family)} WHERE id = $1`, [1]);
}

// A local `undefined` binding is runtime data, not the global undefined value.
export function shadowedUndefined(undefined: string) {
  return read(`SELECT id FROM ${table(undefined)} WHERE id = $1`, [1]);
}
