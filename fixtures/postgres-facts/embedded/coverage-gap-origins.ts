import { query } from "@example/db";

declare const mystery: unknown;

export function missingBinding() {
  return query(mystery);
}

const text = "SELECT 1";

export function throughIdentifier() {
  return query(text);
}

export function brokenHole(flag?: string) {
  return query(`SELECT ${flag ? "a" : mystery}`);
}

export function literalQuery() {
  return query(`SELECT 2`);
}
