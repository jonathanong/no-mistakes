import type { Database as Client } from './barrel';

type CycleA = CycleB;
type CycleB = CycleA;

// Repeated reads share one parameter declaration and one prepared catalog.
export function repeated(db: Client) {
  db.query('select 1');
  db.query('select 2');
  db.query('select 3');
}
export function separate(other: Client) { other.query('select 4'); }
// The same spelling in another lexical scope must have its own cache entry.
export function sameName(db: Client) { db.query('select 5'); }
export function unresolved(missing: Missing) { return missing; }
export function cyclic(cycle: CycleA) { return cycle; }
export function untyped(plain) { return plain; }
