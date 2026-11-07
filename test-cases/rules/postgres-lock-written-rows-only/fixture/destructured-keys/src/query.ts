import { beginTransaction } from "@example/db";
export async function objectParameter({ lockId, writeId }: { lockId: string; writeId: string }) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [lockId]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [writeId]);
}
export async function sameObjectParameter({ id }: { id: string }) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function arrayParameter([lockId, writeId]: string[]) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [lockId]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [writeId]);
}
export async function sameArrayParameter([id]: string[]) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function objectConst(input: { lockId: string; writeId: string }) {
  const { lockId, writeId } = input;
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [lockId]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [writeId]);
}
export async function sameObjectConst(input: { id: string }) {
  const { id } = input;
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function arrayConst(input: string[]) {
  const [lockId, writeId] = input;
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [lockId]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [writeId]);
}
export async function sameArrayConst(input: string[]) {
  const [id] = input;
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
