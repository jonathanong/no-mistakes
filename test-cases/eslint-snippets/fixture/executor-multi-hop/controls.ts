import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Item = { id: string; kind: string };
type LiftInput = { noticeId: string; ids: string[]; items: Item[] };

import { markEach, sharedId } from "./loop-helper";

declare function stash(value: unknown): void;
declare function fetchSummary(id: string): Promise<{ total: number }>;
declare function dynamicHelper(tx: SessionExecutor, id: string): Promise<void>;

let mutable: SessionExecutor | undefined;

async function markItem(tx: SessionExecutor, id: string) {
  await tx(sql`UPDATE items SET marked = true WHERE id = ${id}`);
}

async function storeExecutor(tx: SessionExecutor, id: string) {
  mutable = tx;
  await markItem(tx, id);
}

async function closureExecutor(tx: SessionExecutor, id: string) {
  stash(() => tx);
  await markItem(tx, id);
}

async function reassignExecutor(tx: SessionExecutor, id: string) {
  tx = mutable!;
  await markItem(tx, id);
}

async function dynamicSql(tx: SessionExecutor, id: string) {
  await tx(`UPDATE items SET marked = true WHERE id = ${id}`);
}

async function loopSql(tx: SessionExecutor, input: LiftInput) {
  for (const item of input.items) {
    await tx(sql`UPDATE items SET marked = true WHERE ${item}`);
  }
}

async function loopSqlWrite(tx: SessionExecutor, input: LiftInput) {
  for (const item of input.items) {
    await tx(sql`UPDATE items SET marked = true WHERE id = ${item.id}`);
    item.id = "changed";
  }
}

async function loopHelperSql(tx: SessionExecutor, kind: string) {
  await tx(`UPDATE items SET ${kind} = true`);
}

async function loopHelperCaller(tx: SessionExecutor, input: LiftInput) {
  for (const item of input.items) {
    await loopHelperSql(tx, item.kind);
  }
}

async function unresolvedHop(tx: SessionExecutor, id: string) {
  await dynamicHelper(tx, id);
}

async function escapeHop(tx: SessionExecutor, id: string) {
  stash(tx);
  await markItem(tx, id);
}

async function dynamicHop(tx: SessionExecutor, id: string) {
  await dynamicSql(tx, id);
}

async function cycleA(tx: SessionExecutor, id: string) {
  await cycleB(tx, id);
}

async function cycleB(tx: SessionExecutor, id: string) {
  await cycleA(tx, id);
  await markItem(tx, id);
}

async function spreadHop(args: [SessionExecutor, string]) {
  await markItem(...args);
}

export async function storedCaller(tx: SessionExecutor, id: string) {
  await storeExecutor(tx, id);
}

export async function closureCaller(tx: SessionExecutor, id: string) {
  await closureExecutor(tx, id);
}

export async function reassignCaller(tx: SessionExecutor, id: string) {
  await reassignExecutor(tx, id);
}

export async function dynamicCaller(tx: SessionExecutor, id: string) {
  await dynamicHop(tx, id);
}

export async function loopCaller(tx: SessionExecutor, input: LiftInput) {
  await loopSql(tx, input);
}

export async function loopHelperHop(tx: SessionExecutor, input: LiftInput) {
  await loopHelperCaller(tx, input);
}

export async function unresolvedCaller(tx: SessionExecutor, id: string) {
  await unresolvedHop(tx, id);
}

export async function escapeCaller(tx: SessionExecutor, id: string) {
  await escapeHop(tx, id);
}

export async function cycleCaller(tx: SessionExecutor, id: string) {
  await cycleA(tx, id);
}

async function publishTotal(tx: SessionExecutor, summary: { total: number }) {
  await tx(sql`INSERT INTO summaries (total) VALUES (${summary.total})`);
}

export async function plainResultCaller(tx: SessionExecutor, input: LiftInput) {
  const summary = await fetchSummary(input.noticeId);
  await publishTotal(tx, summary);
}

export async function importedScalarCaller(tx: SessionExecutor) {
  await markItem(tx, sharedId);
}

export async function importedLoopCaller(tx: SessionExecutor, input: LiftInput) {
  await markEach(tx, input.ids);
}

export async function loopWriteCaller(tx: SessionExecutor, input: LiftInput) {
  await loopSqlWrite(tx, input);
}

export async function spreadCaller(tx: SessionExecutor, id: string) {
  await spreadHop([tx, id]);
}
