import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Part = { id: string; owner: string; weight: number; meta: { tag: string } };
type Pair = { a: number; b: string; c: "x" };

async function build(ids: string[], tx: SessionExecutor): Promise<number[]> {
  return tx<number[]>(sql`SELECT count(*) FROM parts WHERE id = ANY(${ids})`);
}

async function deliver(counts: number[], tx: SessionExecutor) {
  await tx(sql`INSERT INTO tallies SELECT * FROM unnest(${counts}::int[])`);
}

async function deliverPairs(pairs: Pair[], tx: SessionExecutor) {
  await tx(sql`INSERT INTO pairs SELECT * FROM jsonb_populate_recordset(null, ${pairs})`);
}

async function reverseInline(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await deliver(await build(rows.map((r) => r.id), tx), tx);
}

async function reverseConst(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  const counts = await build(
    rows.map((r) => r.owner),
    tx,
  );
  await deliver(counts, tx);
}

async function mappedConst(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  const ids = rows.map((r) => r.id);
  await build(ids, tx);
}

async function mappedWeights(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await deliver(
    rows.map((r) => r.weight),
    tx,
  );
}

async function mappedObjects(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await deliverPairs(
    rows.map((r) => ({ a: 1, b: r.id, c: "x" as const })),
    tx,
  );
}

export async function callerOfInline(tx: SessionExecutor) {
  await reverseInline(tx);
}

export async function callerOfConst(tx: SessionExecutor) {
  await reverseConst(tx);
}

export async function callerOfMappedConst(tx: SessionExecutor) {
  await mappedConst(tx);
}

export async function callerOfWeights(tx: SessionExecutor) {
  await mappedWeights(tx);
}

export async function callerOfObjects(tx: SessionExecutor) {
  await mappedObjects(tx);
}
