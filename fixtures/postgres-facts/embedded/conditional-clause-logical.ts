import sql from "sql-template-strings";
import { read } from "@example/db";

const extra = "AND id < $2";

export function orArms(c?: string) {
  const cf = (c ? "AND id > $2" : "") || extra;
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function andArms(c?: string) {
  const cf = (c ? "AND id > $2" : "") && extra;
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function inlineOr(c?: string) {
  return read(
    `SELECT id FROM orders WHERE account_id = $1 ${(c ? "" : "AND id > $2") || "AND id < $2"}`,
    [1],
  );
}

// A second hole that tests the same parameter is its own choice, so the holes multiply.
export function independentSecondHole(c?: string) {
  const cf = (c ? "AND id > $2" : "") || "AND id < $2";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf} ${c ? "/*yes*/" : "/*no*/"}`, [1]);
}

export function nested(c?: string, d?: string) {
  const cf = c ? ((d ? "AND id > $2" : "") || extra) : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

// ?? does not treat "" as missing, so the fallback is not a third alternative.
export function coalesceKeepsEmpty(c?: string) {
  const cf = (c ? "AND id > $2" : "") ?? "AND id < $2";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

// A proven string makes the ?? fallback unreachable, even when that fallback is unknown.
export function coalesceUnknownFallback(c?: string, other?: string) {
  const cf = (c ? "AND id > $2" : "") ?? other;
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

// "" is falsy, so && never evaluates the unknown right.
export function falsyAnd(other?: string) {
  return read(`SELECT id FROM orders WHERE account_id = $1 ${"" && other}`, [1]);
}

// A truthy literal || never evaluates the unknown right.
export function truthyOr(other?: string) {
  const cf = "AND id > $2" || other;
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

// && of a falsy string is stored as an absent fragment. That is not a clause union.
export function constFalsyAnd(other?: string) {
  const cf = "" && other;
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

// A reachable unknown arm abstains. It must not leave only the known arm.
export function unknownOr(c?: string, other?: string) {
  const cf = (c ? "AND id > $2" : other) || "AND id < $2";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function unknownAnd(c?: string) {
  const cf = c && "AND id > $2";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function unknownCoalesce(c?: string) {
  const cf = c ?? "AND id > $2";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function unknownRight(c?: string, other?: string) {
  const cf = (c ? "" : "AND id > $2") || other;
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function sideEffect(c?: string) {
  const q = sql`SELECT id FROM orders WHERE account_id = $1`;
  const cf = q.append(sql` AND id > $2`) || (c ? "AND id > $2" : "");
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

// One empty arm among 16, replaced by a 2-way union, is 17 and must abstain.
export function overCap(a?: string, b?: string, c?: string, d?: string, e?: string) {
  const cf =
    `${a ? "/*a*/" : ""}${b ? "/*b*/" : ""}${c ? "/*c*/" : ""}${d ? "/*d*/" : ""}` ||
    (e ? "/*e1*/" : "/*e0*/");
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}
