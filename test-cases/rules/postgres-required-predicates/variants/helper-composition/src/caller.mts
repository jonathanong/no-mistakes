import sql from "sql-template-strings";
import { read } from "@example/db";
import { buildOuter } from "./outer.mts";
import { levelOne as okOne } from "./chain-ok.mts";
import { levelOne as deepOne } from "./chain-deep.mts";

// Nested helper composition: the view is read three helpers down, and the
// alias arguments are string literals that are spliced into raw appends.
export async function composed(id: string) {
  const query = sql`SELECT p.id FROM plans p JOIN roots r ON r.id = p.root_id WHERE p.owner = ${id} AND `;
  query.append(buildOuter("p", "r"));
  return read(query);
}

// The same helper with an options object that only feeds branch tests.
export async function withOptions(id: string) {
  const query = sql`SELECT p.id FROM plans p JOIN roots r ON r.id = p.root_id WHERE p.owner = ${id} AND `;
  query.append(buildOuter("p", "r", { strict: true, includeDrafts: false }));
  return read(query);
}

// A helper result bound to a `const` in one branch and appended later.
export async function boundInBranch(id: string, all: boolean) {
  const query = sql`SELECT p.id FROM plans p JOIN roots r ON r.id = p.root_id WHERE p.owner = ${id}`;
  if (all) {
    query.append(sql` AND TRUE`);
  } else {
    const eligibility = buildOuter("p", "r", { includeDrafts: true });
    query.append(sql` AND `);
    query.append(eligibility).append(sql` ORDER BY p.id`);
  }
  return read(query);
}

// Eight helpers deep is the limit and still recovers.
export async function eightDeep() {
  return read(okOne());
}

// A ninth helper is past the limit, so the read stays unanalyzable.
export async function nineDeep() {
  return read(deepOne());
}
