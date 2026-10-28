import { query } from '@example/db';
declare const it: { each(rows: readonly string[]): (name: string, fn: (field: string) => void) => void };
it.each(['safe', 'bad'])('untrusted runner', field => {
  const updates = { safe: 'SELECT 1', bad: 'DROP TABLE app.fake_runner' };
  query(updates[field]); // residual fake runner
});
