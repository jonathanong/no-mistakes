import { query, read, sql } from "@example/db";

export function nullByte() {
  return read(`SELECT \0 ${"a"}`);
}

export function numericHole() {
  return read(`SELECT ${1.0}`);
}

export function escapedNewline() {
  return read(`SELECT ${"\n"}`);
}

export function nestedFlag(flag?: string) {
  return read(`SELECT ${flag ? (flag ? "a" : "b") : (flag ? "c" : "d")}`);
}

export function constrainedPiece(flag?: string) {
  const piece = flag ? "a" : "b";
  return read(`SELECT ${flag ? piece : "c"}`);
}

export function mutatingTest() {
  const builder = sql`SELECT 1`;
  return read(`SELECT ${builder.append(sql`x`) ? "a" : "b"}`);
}

function marked(kind: string) {
  switch (kind) {
    case "a":
      return "\u{10FFFE}";
    default:
      throw new Error("no");
  }
}

export function markedHole(kind: string) {
  return read(`SELECT ${marked(kind)}`);
}

function plainTable(kind: string) {
  switch (kind) {
    case "a":
      return "orders";
    default:
      throw new Error("no");
  }
}

export function plainHole(kind: string) {
  return read(`SELECT ${plainTable(kind)}`);
}

export function spreadBuilderValues(
  builder: { text: string; values: unknown[] },
  extra: unknown[],
) {
  return read(`${builder.text}`, ...extra);
}

export function literalRead() {
  return read(`SELECT 1`);
}

const named = { orders: "orders" };

export function stringKey() {
  return read(`SELECT ${named.orders}`);
}

const numbered = { 1: "orders" };

export function numericKey() {
  return read(`SELECT ${numbered[1]}`);
}

export function deepTemplate() {
  return read(
    `SELECT ${`${`${`${`${`${`${`${`${`${"a"}`}`}`}`}`}`}`}`}`}`,
  );
}

export function guardedTables(table: string) {
  const allowed = new Set(["orders", "accounts"]);
  if (!allowed.has(table)) throw new Error("no");
  return query(sql`SELECT `.append(table));
}

export function spreadConstructor(table: string, extra: string[]) {
  const allowed = new Set(...extra);
  if (!allowed.has(table)) throw new Error("no");
  return query(sql`SELECT `.append(table));
}

export function emptyElement(table: string) {
  const allowed = new Set([""]);
  if (!allowed.has(table)) throw new Error("no");
  return query(sql`SELECT `.append(table));
}

export function missingHasArgument(table: string) {
  const allowed = new Set(["orders"]);
  if (!allowed.has()) throw new Error("no");
  return query(sql`SELECT `.append(table));
}

export function spreadHasArgument(table: string) {
  const allowed = new Set(["orders"]);
  if (!allowed.has(...table)) throw new Error("no");
  return query(sql`SELECT `.append(table));
}

export function bareAppend() {
  return read(sql`SELECT 1`.append());
}

export function spreadAppend(extra: string[]) {
  return read(sql`SELECT 1`.append(...extra));
}
