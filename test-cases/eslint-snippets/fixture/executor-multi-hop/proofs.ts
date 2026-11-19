import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Item = { id: string; kind: string };
type LiftInput = { noticeId: string; ids: string[]; items: Item[] };
type Summary = { total: number };

async function markItem(tx: SessionExecutor, id: string) {
  await tx(sql`UPDATE items SET marked = true WHERE id = ${id}`);
}

async function recordAudit(tx: SessionExecutor, noticeId: string) {
  await tx(sql`INSERT INTO audits (notice_id) VALUES (${noticeId})`);
}

async function loadSummary(tx: SessionExecutor, noticeId: string): Promise<Summary> {
  return (await tx(
    sql`SELECT count(*) AS total FROM items WHERE notice_id = ${noticeId}`,
  )) as Summary;
}

async function publishSummary(tx: SessionExecutor, summary: Summary) {
  await tx(sql`INSERT INTO summaries (total) VALUES (${summary.total})`);
}

async function lockAndMark(tx: SessionExecutor, id: string) {
  await markItem(tx, id);
  await tx(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
}

async function outerHop(tx: SessionExecutor, id: string) {
  await lockAndMark(tx, id);
}

async function loops(tx: SessionExecutor, input: LiftInput) {
  for (const item of input.items) {
    await markItem(tx, item.id);
  }
  for (const id of input.ids) {
    await markItem(tx, id);
  }
  for (let index = 0; index < input.items.length; index += 1) {
    await markItem(tx, input.noticeId);
  }
}

async function sequential(tx: SessionExecutor, input: LiftInput) {
  await markItem(tx, input.noticeId);
  await recordAudit(tx, input.noticeId);
}

async function awaitedResult(tx: SessionExecutor, input: LiftInput) {
  const summary = await loadSummary(tx, input.noticeId);
  await publishSummary(tx, summary);
}

async function loopField(tx: SessionExecutor, input: LiftInput) {
  for (const item of input.items) {
    await tx(sql`UPDATE items SET marked = true WHERE id = ${item.id}`);
  }
}

async function nestedHops(tx: SessionExecutor, input: LiftInput) {
  await outerHop(tx, input.noticeId);
}

async function applyLift(tx: SessionExecutor, input: LiftInput) {
  for (const item of input.items) {
    await markItem(tx, item.id);
  }
  await recordAudit(tx, input.noticeId);
  const summary = await loadSummary(tx, input.noticeId);
  await publishSummary(tx, summary);
}

export async function callerOfLift(tx: SessionExecutor, input: LiftInput) {
  await applyLift(tx, input);
}

export async function callerOfLoops(tx: SessionExecutor, input: LiftInput) {
  await loops(tx, input);
}

export async function callerOfSequential(tx: SessionExecutor, input: LiftInput) {
  await sequential(tx, input);
}

export async function callerOfAwaited(tx: SessionExecutor, input: LiftInput) {
  await awaitedResult(tx, input);
}

export async function callerOfLoopField(tx: SessionExecutor, input: LiftInput) {
  await loopField(tx, input);
}

export async function callerOfNested(tx: SessionExecutor, input: LiftInput) {
  await nestedHops(tx, input);
}
