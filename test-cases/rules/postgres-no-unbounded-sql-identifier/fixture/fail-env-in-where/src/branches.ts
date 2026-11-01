import { query } from "@example/db";

export function ternaryFiniteFirst(flag: boolean, uid: string) {
  const cfg = { safe: "users", column: process.env.BCOL1 };
  const tbl = flag ? cfg.safe : cfg.column;
  return query(`SELECT id FROM ${tbl} WHERE id = ${uid}`);
}

export function ternaryUnboundedFirst(flag: boolean, uid: string) {
  const cfg = { safe: "users", column: process.env.BCOL2 };
  const tbl = flag ? cfg.column : cfg.safe;
  return query(`SELECT id FROM ${tbl} WHERE id = ${uid}`);
}

export function nullishFiniteFirst(uid: string) {
  const cfg = { safe: "users", column: process.env.BCOL3 };
  const tbl = cfg.safe ?? cfg.column;
  return query(`SELECT id FROM ${tbl} WHERE id = ${uid}`);
}

export function nullishUnboundedFirst(uid: string) {
  const cfg = { safe: "users", column: process.env.BCOL4 };
  const tbl = cfg.column ?? cfg.safe;
  return query(`SELECT id FROM ${tbl} WHERE id = ${uid}`);
}
