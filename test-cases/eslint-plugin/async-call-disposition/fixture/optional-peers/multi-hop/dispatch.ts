import { enqueueBatch, computeDelay } from "@app/jobs";
import { drainGroups, drainDropped } from "@app/handler";
import { processBatches, sql } from "./batching";

type EnqueueBatch = typeof enqueueBatch;
export type Dependencies = {
  now?: Date;
  enqueueBatch?: EnqueueBatch;
  computeDelay?: typeof computeDelay;
};
type Options = Dependencies & { key?: string; itemIds?: readonly string[] };

export async function dispatch(options: Options = {}) {
  const clock = (options.now ?? new Date()).getTime();
  void clock;
  // Tagged query syntax must not hide an otherwise provable callback chain.
  const constantQuery = sql`SELECT 1`;
  const escapedQuery = sql`SELECT \`item\`\n`;
  // Invalid escapes are legal in tagged templates; their cooked value is null.
  const rawQuery = sql`\unicode`;
  void constantQuery;
  void escapedQuery;
  void rawQuery;
  const query = sql`SELECT item_id FROM items WHERE group_key = ${options.key}`;
  await processBatches(query, {
    handler: async rows => {
      await drainGroups(rows, {
        enqueueBatch: options.enqueueBatch ?? enqueueBatch,
        computeDelay: options.computeDelay ?? computeDelay,
      });
    },
  });
}

export async function dispatchDropped(options: Dependencies) {
  await drainDropped([], {
    enqueueBatch: options.enqueueBatch ?? enqueueBatch,
    computeDelay: options.computeDelay ?? computeDelay,
  });
}

// A later awaited path cannot repair the earlier invocation as a template tag.
export async function dispatchTagged(options: Dependencies) {
  options.enqueueBatch`item`;
  await dispatch(options);
}

// Substitutions execute too: their callback promises must not disappear in conversion.
export async function dispatchInterpolated(options: Dependencies) {
  const query = sql`SELECT ${options.enqueueBatch([], {})}`;
  void query;
  await dispatch(options);
}

export async function dispatchPlainInterpolated(options: Dependencies) {
  const query = `SELECT ${options.enqueueBatch([], {})}`;
  void query;
  await dispatch(options);
}
