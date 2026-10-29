import { read, write } from '@example/db';
import * as db from '@example/db';

// Positive controls: a reference is not a call, but calling the alias still reports.
const run = write;
run('SELECT 1');

export function aliases(options: any = db) {
  const viaNullish = options.query ?? write;
  viaNullish('SELECT 2');
  const viaOr = options.query || write;
  viaOr('SELECT 3');
  const viaTernary = options.query ? write : read;
  viaTernary('SELECT 4');
  const viaCast = options.query as Function;
  viaCast('SELECT 5');
  (0, options.query)('SELECT 6');
}
