import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };

declare function stash(notice: Notice): void;
declare function dynamicId(): string;
declare const key: "id";

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

export async function mutatedBeforeDestructure(
  query: SessionExecutor,
  rows: Notice[],
  next: string,
) {
  const notice = rows[0]!;
  notice.id = next;
  const { id } = notice;
  return lockNotice(query, id);
}

export async function chainedWritten(query: SessionExecutor, rows: Notice[], next: string) {
  const first = rows[0]!;
  first.id = next;
  const notice = first;
  return lockNotice(query, notice.id);
}

export async function chainedLet(query: SessionExecutor, rows: Notice[]) {
  let first = rows[0]!;
  first = rows[1]!;
  const notice = first;
  return lockNotice(query, notice.id);
}

export async function unknownDestructure(query: SessionExecutor) {
  const { id } = { id: dynamicId() };
  return lockNotice(query, id);
}

export async function mutatedObject(query: SessionExecutor, next: string) {
  const notice = { id: "a", status: "b" };
  notice.id = next;
  return lockNotice(query, notice.id);
}

export async function escapedObject(query: SessionExecutor) {
  const notice = { id: "a", status: "b" };
  stash(notice);
  return lockNotice(query, notice.id);
}

export async function letObject(query: SessionExecutor, next: string) {
  let notice = { id: "a", status: "b" };
  notice = { id: next, status: "b" };
  return lockNotice(query, notice.id);
}

export async function computedKey(query: SessionExecutor) {
  const notice = { [key]: dynamicId(), status: "b" };
  return lockNotice(query, notice.id);
}

export async function spreadObject(query: SessionExecutor, rows: Notice[]) {
  const notice = { ...rows[0]!, status: "b" };
  return lockNotice(query, notice.id);
}

export async function dynamicField(query: SessionExecutor) {
  const notice = { id: dynamicId(), status: "b" };
  return lockNotice(query, notice.id);
}
