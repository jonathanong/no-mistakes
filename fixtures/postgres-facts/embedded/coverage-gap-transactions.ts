import { beginTransaction } from "@example/db";

export async function takenTrue() {
  const tx = await beginTransaction();
  if (true) await tx.query(`/* takenTrue */ SELECT 1`);
  else await tx.query(`/* takenTrueElse */ SELECT 2`);
}

export async function takenFalse() {
  const tx = await beginTransaction();
  if (false) await tx.query(`/* takenFalse */ SELECT 1`);
  else await tx.query(`/* takenFalseElse */ SELECT 2`);
}

export async function unknownThrow(flag?: boolean) {
  const tx = await beginTransaction();
  if (flag) throw new Error("stop");
  else await tx.query(`/* unknownThrow */ SELECT 1`);
}

export async function unknownElseThrow(flag?: boolean) {
  const tx = await beginTransaction();
  if (flag) await tx.query(`/* unknownElseThrow */ SELECT 1`);
  else throw new Error("stop");
}

export async function unknownJoin(flag?: boolean) {
  const tx = await beginTransaction();
  if (flag) await tx.query(`/* unknownJoin */ SELECT 1`);
  else await tx.query(`/* unknownJoinElse */ SELECT 2`);
}

export function goodLock(tx: { query(text: string): unknown }) {
  return tx.query(`SELECT 1`);
}

export function missingSql(tx: { query(): unknown }) {
  return tx.query();
}

export function callableLock(tx: (text: string) => unknown) {
  return tx(`SELECT 1`);
}

export function missingArgument() {
  return goodLock();
}
