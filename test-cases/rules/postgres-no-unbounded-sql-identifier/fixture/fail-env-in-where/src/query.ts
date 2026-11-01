import { query } from "@example/db";

export function whereColumn(table: string, uid: string) {
  return query(`SELECT id FROM ${table} WHERE ${process.env.WCOL} = ${uid}`);
}

export function whereAlias(table: string, uid: string) {
  const col = process.env.WCOL;
  return query(`SELECT id FROM ${table} WHERE ${col} = ${uid}`);
}

export function joinOn(table: string, uid: string) {
  return query(`SELECT id FROM ${table} JOIN o ON o.x = ${process.env.JCOL} AND o.id = ${uid}`);
}

export function functionArgument(table: string, uid: string) {
  return query(`SELECT lower(${process.env.FCOL}) FROM ${table} WHERE id = ${uid}`);
}

export function having(table: string, uid: string) {
  return query(`SELECT count(*) FROM ${table} GROUP BY id HAVING ${process.env.HCOL} > ${uid}`);
}

export function returning(table: string, uid: string) {
  return query(`UPDATE ${table} SET name = ${uid} RETURNING ${process.env.RCOL}`);
}

export function conflict(table: string, uid: string) {
  return query(`INSERT INTO ${table} (id) VALUES (${uid}) ON CONFLICT (${process.env.CCOL}) DO NOTHING`);
}

export function subquery(table: string, uid: string) {
  return query(`SELECT id FROM ${table} WHERE a IN (SELECT ${process.env.SCOL} FROM t2 WHERE id = ${uid})`);
}
