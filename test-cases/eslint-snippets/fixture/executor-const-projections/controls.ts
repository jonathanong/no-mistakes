import assert from "fake-assert";
import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };

declare function mutate(notice: Notice): void;
declare function stash(notice: Notice): void;
declare function unknownRow(): any;
declare function lookup(): Notice;
declare function lookupRows(): Promise<{ rows: Notice[] }>;
declare const sink: Notice[];

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

export async function letLocal(query: SessionExecutor, rows: Notice[]) {
  let notice = rows[0]!;
  return lockNotice(query, notice.id);
}

export async function reassigned(query: SessionExecutor, rows: Notice[], other: Notice) {
  let notice = rows[0]!;
  notice = other;
  return lockNotice(query, notice.id);
}

export async function fieldWritten(query: SessionExecutor, rows: Notice[], next: string) {
  const notice = rows[0]!;
  notice.id = next;
  return lockNotice(query, notice.id);
}

export async function sourceWritten(query: SessionExecutor, rows: Notice[], next: string) {
  const notice = rows[0]!;
  rows[0]!.id = next;
  return lockNotice(query, notice.id);
}

export async function anyInit(query: SessionExecutor) {
  const notice = unknownRow();
  return lockNotice(query, notice.id);
}

export async function unknownCall(query: SessionExecutor) {
  const notice = lookup();
  return lockNotice(query, notice.id);
}

export async function writingCallee(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  mutate(notice);
  return lockNotice(query, notice.id);
}

export async function escaped(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  sink.push(notice);
  return lockNotice(query, notice.id);
}

export async function escapedCall(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  stash(notice);
  return lockNotice(query, notice.id);
}

export async function shadowed(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  {
    const notice = lookup();
    return lockNotice(query, notice.id);
  }
}

export async function otherSource(query: SessionExecutor) {
  const { rows } = await lookupRows();
  const notice = rows[0]!;
  assert(notice, 404, "not found");
  return lockNotice(query, notice.id);
}
