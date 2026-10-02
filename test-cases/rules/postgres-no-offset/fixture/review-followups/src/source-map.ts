import { query } from '@data-stores/psql';
const shared = "SELECT id FROM orders\nOFFSET 0";
export const declaration = () => query(shared);
export const multiline = () => query(
  `SELECT id FROM orders
   WHERE id = ${
      1
   } OFFSET 1`);
export const cooked = () => query(`SELECT id FROM orders\nOFFSET 1`);
export const continuation = () => query(`SELECT id FROM orders \
OFFSET 1`);
export const raw = () => query(String.raw`SELECT '\\n' FROM orders
OFFSET 1`);
const below =
  `SELECT id FROM orders
   OFFSET 1`;
export const belowCall = () => query(below);
// no-mistakes-disable-next-line postgres-no-offset
export const suppressedCooked = () => query(`SELECT id FROM orders\nOFFSET 1`);
export const suppressedInterpolation = () => query(`SELECT id FROM orders WHERE id = ${
  1
} OFFSET 1`); // no-mistakes-disable-line postgres-no-offset
