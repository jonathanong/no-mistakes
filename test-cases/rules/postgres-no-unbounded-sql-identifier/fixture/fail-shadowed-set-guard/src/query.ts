import { query } from "@example/db";

function trusted(value: string) {
  const names = new Set(["users"]);
  if (!names.has(value)) throw new Error();
  return value;
}

function shadowed(value: string, Set: any) {
  const names = new Set(["users"]);
  if (!names.has(value)) throw new Error();
  return value;
}

function caught(value: string) {
  const names = new Set(["users"]);
  if (!names.has(value)) throw new Error();
  try {
    return value;
  } catch (Set) {
    throw new Error();
  }
}

export function trustedQuery(value: string) {
  return query(`SELECT id FROM ${trusted(value)}`);
}

export function shadowedQuery(value: string) {
  return query(`SELECT id FROM ${shadowed(value)}`);
}

export function caughtQuery(value: string) {
  return query(`SELECT id FROM ${caught(value)}`);
}

export function shadowedInline(value: string, Set: any) {
  const names = new Set(["users"]);
  if (!names.has(value)) throw new Error();
  return query(`SELECT id FROM ${value}`);
}
