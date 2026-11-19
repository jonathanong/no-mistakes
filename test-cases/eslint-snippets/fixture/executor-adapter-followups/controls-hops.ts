import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

async function h9(tx: SessionExecutor, id: string) {
  await tx(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}
async function h8(tx: SessionExecutor, id: string) {
  await h9(tx, id);
}
async function h7(tx: SessionExecutor, id: string) {
  await h8(tx, id);
}
async function h6(tx: SessionExecutor, id: string) {
  await h7(tx, id);
}
async function h5(tx: SessionExecutor, id: string) {
  await h6(tx, id);
}
async function h4(tx: SessionExecutor, id: string) {
  await h5(tx, id);
}
async function h3(tx: SessionExecutor, id: string) {
  await h4(tx, id);
}
async function h2(tx: SessionExecutor, id: string) {
  await h3(tx, id);
}
async function h1(tx: SessionExecutor, id: string) {
  await h2(tx, id);
}

export async function nineHops(tx: SessionExecutor, id: string) {
  await h1(tx, id);
}
