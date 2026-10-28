import { it } from 'vitest';
import { query } from '@example/db';
declare const fakeRunner: typeof it;
function run() {
  const it = fakeRunner;
  it.each(['safe', 'bad'])('shadowed runner', field => {
    const updates = { safe: 'SELECT 1', bad: 'DROP TABLE app.shadowed_runner' };
    query(updates[field]); // residual shadowed runner
  });
}
run();
