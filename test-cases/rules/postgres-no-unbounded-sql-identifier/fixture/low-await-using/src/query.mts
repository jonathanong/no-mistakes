import { beginTransaction, read, updateThing, write } from '@example/db';
import type { QueryOptions } from '@example/types';
import sql from 'sql-template-strings';
import { updateThing as storeUpdate } from './store.mjs';

declare const relation: string;
declare const id: string;
declare const thingId: string;
declare const column: string;
declare const value: string;
declare const options: QueryOptions;
declare const deps: {
  updateThingImpl?: (id: string, patch: { col: string }) => Promise<void>;
};

export async function awaitUsingArrow() {
  await using tx = await beginTransaction();
  const inner = async (q: typeof tx) => {
    await q(`/* x */ DELETE FROM things WHERE id = $1`, [id]);
    const { rows } = await q(
      sql`/* y */ UPDATE things SET col = ${column} WHERE id = ${value} RETURNING id`,
    );
    await q(`SELECT * FROM ${relation}`);
    return rows;
  };
  return inner(tx);
}

export async function awaitUsingFunction() {
  await using tx = await beginTransaction();
  async function inner(q: typeof tx) {
    await q(`/* x */ DELETE FROM things WHERE id = $1`, [id]);
    await q(sql`/* y */ UPDATE things SET col = ${column} WHERE id = ${value} RETURNING id`);
    await q(`SELECT * FROM ${relation}`);
  }
  await inner(tx);
}

export function usingBinding() {
  using tx = beginTransaction();
  function inner(q: typeof tx) {
    q(`SELECT id FROM things WHERE id = $1`, [id]);
    q(`SELECT * FROM ${relation}`);
  }
  inner(tx);
}

export async function injectedHelpers() {
  const updateThingFn = deps.updateThingImpl ?? updateThing;
  await updateThingFn(thingId, { col: 'v' });
  const storeFn = deps.updateThingImpl ?? storeUpdate;
  await storeFn(thingId, { col: 'v' });
  const fromField = options.updateThingImpl ?? updateThing;
  await fromField(thingId, { col: 'v' });
}

export async function executorChoice() {
  const run = options.query ?? read;
  await run(`SELECT 1`);
  await run(sql`SELECT id FROM users WHERE id = ${id}`);
  await run(`SELECT * FROM ${relation}`);
  const writeRun = options.query ?? write;
  await writeRun(`DELETE FROM users WHERE id = $1`, [id]);
  await writeRun(`DELETE FROM ${relation} WHERE id = $1`, [id]);
  const mixed = options.query ?? updateThing;
  await mixed(`SELECT * FROM ${relation}`);
}
