import { read } from "@example/db";

const extra = "AND id > $2";

export function throughConst(c?: string) {
  const cf = c ? "AND id > $2" : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function inline(c?: string) {
  return read(`SELECT id FROM orders WHERE account_id = $1 ${c ? "AND id > $2" : ""}`, [1]);
}

export function splicedConst(c?: string) {
  const cf = c ? `${extra}` : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function immutableConst(c?: string) {
  const cf = c ? extra : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function nested(c?: string, d?: string) {
  const cf = c ? (d ? "AND id > $2" : "AND id < $2") : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}
