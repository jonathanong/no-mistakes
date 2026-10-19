import { read } from "@example/db";

const known = "AND id > $2";
const alias = known;

// An unknown arm must not leave only the known arm in the union.
export function parameterArm(c?: string, extra?: string) {
  const cf = c ? "AND id > $2" : extra;
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function callArm(c?: string) {
  return read(`SELECT id FROM orders WHERE account_id = $1 ${c ? "AND id > $2" : clause()}`, [1]);
}

function clause() {
  return "AND id > $2";
}

export function aliasArm(c?: string) {
  const cf = c ? alias : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function letArm(c?: string) {
  let extra = "AND id > $2";
  const cf = c ? extra : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function reassigned(c?: string) {
  let extra = "AND id > $2";
  extra = "";
  const cf = c ? extra : "AND id > $2";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function switched(c?: string) {
  let clause = "";
  switch (c) {
    case "a":
      clause = "AND id > $2";
      break;
    default:
      clause = "";
      break;
  }
  return read(`SELECT id FROM orders WHERE account_id = $1 ${clause}`, [1]);
}

async function built() {
  return "AND id > $2";
}

export function asyncArm(c?: string) {
  const cf = c ? "AND id > $2" : built();
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

// A member is not a literal const splice inside a ternary arm.
export function container(c?: string) {
  const box = { clause: "AND id > $2" };
  const cf = c ? box.clause : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function logical(c?: string) {
  const cf = c && "AND id > $2";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}

export function nestedUnknown(c?: string, d?: string, extra?: string) {
  const cf = c ? (d ? "AND id > $2" : extra) : "";
  return read(`SELECT id FROM orders WHERE account_id = $1 ${cf}`, [1]);
}
