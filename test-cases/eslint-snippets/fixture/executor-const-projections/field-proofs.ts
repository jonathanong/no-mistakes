import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

export async function destructured(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  const { id } = notice;
  return lockNotice(query, id);
}

export async function renamedDestructure(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  const { id: noticeId } = notice;
  return lockNotice(query, noticeId);
}

export async function chained(query: SessionExecutor, rows: Notice[]) {
  const first = rows[0]!;
  const notice = first;
  return lockNotice(query, notice.id);
}

export async function constObject(query: SessionExecutor) {
  const notice = { id: "a", status: "b" };
  return lockNotice(query, notice.id);
}

export async function annotatedConstObject(query: SessionExecutor) {
  const notice: Notice = { id: "a", status: "b" };
  return lockNotice(query, notice.id);
}

declare function consume(notice: Notice): void;
declare const anyConsumer: any;
declare function unknownConsumer(notice: unknown): unknown;

export async function consumedAfterDestructure(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  const { id } = notice;
  await lockNotice(query, id);
  consume(notice);
}

export async function anyAfter(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  const { id } = notice;
  const locked = lockNotice(query, id);
  anyConsumer(notice);
  return locked;
}

export async function unknownAfter(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  const { id } = notice;
  const locked = lockNotice(query, id);
  unknownConsumer(notice);
  return locked;
}

export async function writtenAfterDestructure(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  const { id } = notice;
  notice.id = "changed";
  return lockNotice(query, id);
}
