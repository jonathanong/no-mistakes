import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Part = { id: string; owner: string; weight: number; meta: { tag: string } };

declare function stash(value: unknown): void;
declare function fetchCounts(id: string): Promise<number[]>;
declare const unrelated: Part[];

async function build(ids: unknown, tx: SessionExecutor): Promise<number[]> {
  return tx<number[]>(sql`SELECT count(*) FROM parts WHERE id = ANY(${ids})`);
}

async function buildDynamic(ids: unknown, tx: SessionExecutor): Promise<number[]> {
  return tx<number[]>(`SELECT count(*) FROM parts WHERE id = ANY(${ids})`);
}

async function deliver(counts: unknown, tx: SessionExecutor) {
  await tx(sql`INSERT INTO tallies SELECT * FROM unnest(${counts}::int[])`);
}

async function deliverDynamic(counts: unknown, tx: SessionExecutor) {
  await tx(`INSERT INTO tallies SELECT * FROM unnest(${counts}::int[])`);
}

async function claim(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  return rows;
}

async function blockCallback(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => {
      return r.id;
    }),
    tx,
  );
}

async function sideEffect(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => stash(r)),
    tx,
  );
}

async function sequenceBody(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => (stash(r), r.id)),
    tx,
  );
}

async function wholeRow(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => r),
    tx,
  );
}

async function spreadRow(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => ({ ...r })),
    tx,
  );
}

async function wholeRowField(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => ({ a: 1, b: r })),
    tx,
  );
}

async function getterField(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => ({
      get a() {
        return r.id;
      },
    })),
    tx,
  );
}

async function computedField(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => ({ [r.id]: 1 })),
    tx,
  );
}

async function mutatedMapped(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  const ids = rows.map((r) => r.id);
  ids.push("extra");
  await build(ids, tx);
}

async function reassignedMapped(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  let ids = rows.map((r) => r.id);
  ids = [];
  await build(ids, tx);
}

async function mutatedRows(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  rows.push(rows[0]);
  await build(
    rows.map((r) => r.id),
    tx,
  );
}

async function nonScalarField(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => r.meta),
    tx,
  );
}

async function nonScalarObjectField(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => ({ a: r.meta })),
    tx,
  );
}

async function unprovenResult(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  const counts = await buildDynamic(
    rows.map((r) => r.id),
    tx,
  );
  await deliver(counts, tx);
}

async function unprovenInline(tx: SessionExecutor) {
  await deliver(await buildDynamic(["fixed"], tx), tx);
}

async function foreignResult(tx: SessionExecutor, id: string) {
  const counts = await fetchCounts(id);
  await deliver(counts, tx);
}

async function dynamicHop(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await deliverDynamic(await build(["fixed"], tx), tx);
}

async function otherReceiver(tx: SessionExecutor, parts: Part[]) {
  await build(
    parts.map((r) => r.id),
    tx,
  );
}

async function memberReceiver(tx: SessionExecutor, input: { parts: Part[] }) {
  await build(
    input.parts.map((r) => r.id),
    tx,
  );
}

async function ambientReceiver(tx: SessionExecutor) {
  await build(
    unrelated.map((r) => r.id),
    tx,
  );
}

async function unresolvedReceiver(tx: SessionExecutor) {
  await build(
    missingRows.map((r) => r.id),
    tx,
  );
}

async function returnedRows(tx: SessionExecutor) {
  const rows = await claim(tx);
  await build(
    rows.map((r) => r.id),
    tx,
  );
}

async function otherMethod(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.filter((r) => r.id),
    tx,
  );
}

async function computedMethod(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows["map"]((r) => r.id),
    tx,
  );
}

async function optionalMethod(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows?.map((r) => r.id),
    tx,
  );
}

async function optionalCall(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map?.((r) => r.id),
    tx,
  );
}

async function plainCall(tx: SessionExecutor) {
  await build(stash(tx), tx);
}

async function functionCallback(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map(function (r) {
      return r.id;
    }),
    tx,
  );
}

async function extraMapArgument(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r) => r.id, stash),
    tx,
  );
}

async function indexedCallback(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map((r, index) => r.id + index),
    tx,
  );
}

async function destructuredCallback(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map(({ id }) => id),
    tx,
  );
}

async function asyncCallback(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  await build(
    rows.map(async (r) => r.id),
    tx,
  );
}

async function destructuredDerived(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Part[] }>(sql`UPDATE parts SET done = true RETURNING id`);
  const [first] = rows.map((r) => r.meta);
  await build(first, tx);
}

async function emptyDerived(tx: SessionExecutor) {
  let ids: string[];
  await build(ids, tx);
}

export async function callerOfBlock(tx: SessionExecutor) {
  await blockCallback(tx);
}

export async function callerOfSideEffect(tx: SessionExecutor) {
  await sideEffect(tx);
}

export async function callerOfSequence(tx: SessionExecutor) {
  await sequenceBody(tx);
}

export async function callerOfWholeRow(tx: SessionExecutor) {
  await wholeRow(tx);
}

export async function callerOfSpread(tx: SessionExecutor) {
  await spreadRow(tx);
}

export async function callerOfWholeRowField(tx: SessionExecutor) {
  await wholeRowField(tx);
}

export async function callerOfGetter(tx: SessionExecutor) {
  await getterField(tx);
}

export async function callerOfComputed(tx: SessionExecutor) {
  await computedField(tx);
}

export async function callerOfMutated(tx: SessionExecutor) {
  await mutatedMapped(tx);
}

export async function callerOfReassigned(tx: SessionExecutor) {
  await reassignedMapped(tx);
}

export async function callerOfMutatedRows(tx: SessionExecutor) {
  await mutatedRows(tx);
}

export async function callerOfNonScalar(tx: SessionExecutor) {
  await nonScalarField(tx);
}

export async function callerOfNonScalarObject(tx: SessionExecutor) {
  await nonScalarObjectField(tx);
}

export async function callerOfUnproven(tx: SessionExecutor) {
  await unprovenResult(tx);
}

export async function callerOfUnprovenInline(tx: SessionExecutor) {
  await unprovenInline(tx);
}

export async function callerOfForeign(tx: SessionExecutor, id: string) {
  await foreignResult(tx, id);
}

export async function callerOfDynamicHop(tx: SessionExecutor) {
  await dynamicHop(tx);
}

export async function callerOfOtherReceiver(tx: SessionExecutor, parts: Part[]) {
  await otherReceiver(tx, parts);
}

export async function callerOfMemberReceiver(tx: SessionExecutor, input: { parts: Part[] }) {
  await memberReceiver(tx, input);
}

export async function callerOfAmbient(tx: SessionExecutor) {
  await ambientReceiver(tx);
}

export async function callerOfUnresolved(tx: SessionExecutor) {
  await unresolvedReceiver(tx);
}

export async function callerOfReturnedRows(tx: SessionExecutor) {
  await returnedRows(tx);
}

export async function callerOfOtherMethod(tx: SessionExecutor) {
  await otherMethod(tx);
}

export async function callerOfComputedMethod(tx: SessionExecutor) {
  await computedMethod(tx);
}

export async function callerOfOptionalMethod(tx: SessionExecutor) {
  await optionalMethod(tx);
}

export async function callerOfOptionalCall(tx: SessionExecutor) {
  await optionalCall(tx);
}

export async function callerOfPlainCall(tx: SessionExecutor) {
  await plainCall(tx);
}

export async function callerOfFunctionCallback(tx: SessionExecutor) {
  await functionCallback(tx);
}

export async function callerOfExtraArgument(tx: SessionExecutor) {
  await extraMapArgument(tx);
}

export async function callerOfIndexed(tx: SessionExecutor) {
  await indexedCallback(tx);
}

export async function callerOfDestructured(tx: SessionExecutor) {
  await destructuredCallback(tx);
}

export async function callerOfAsync(tx: SessionExecutor) {
  await asyncCallback(tx);
}

export async function callerOfDestructuredDerived(tx: SessionExecutor) {
  await destructuredDerived(tx);
}

export async function callerOfEmpty(tx: SessionExecutor) {
  await emptyDerived(tx);
}
