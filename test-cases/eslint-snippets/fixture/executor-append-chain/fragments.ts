import sql from "@example/sql";

export const SELECT_COLS = sql`id, state`;
export const JOINS = sql` JOIN owners ON owners.id = widgets.owner_id`;
export const MUTATED = sql`id, state`;
export const PARAMETERIZED = sql`id, ${"state"}`;
export let LET_COLS = sql`id, state`;
export const CALLED = sql`id`.append(sql`, state`);
