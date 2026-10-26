import { query, type QueryExecutor } from '@example/db';

// Keep identical SQL at named and typed executors: allowing executor passing
// must never exempt the receiving helper's SQL from the configured policies.
export function named() {
  query('CREATE TABLE forbidden (id integer)');
  query('SELECT id FROM orders OFFSET 1');
}

export function typed(executor: QueryExecutor) {
  executor('CREATE TABLE forbidden (id integer)');
  executor('SELECT id FROM orders OFFSET 1');
}

export function member(executor: QueryExecutor) {
  executor.query('CREATE TABLE forbidden (id integer)');
  executor.query('SELECT id FROM orders OFFSET 1');
}

export function passed() {
  typed(query);
  member(query);
}

export function destructured({ run: executor }: { run: QueryExecutor }) {
  executor('CREATE TABLE forbidden (id integer)');
  executor('SELECT id FROM orders OFFSET 1');
}

export function writes(executor: QueryExecutor) {
  query('/* named/write */ UPDATE orders SET doubled = 1');
  executor('/* typed/write */ UPDATE orders SET doubled = 1');
  executor.query('/* member/write */ UPDATE orders SET doubled = 1');
}
