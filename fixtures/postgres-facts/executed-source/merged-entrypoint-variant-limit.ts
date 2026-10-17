import { query } from '@example/db';

function execute(sql: string) {
  return query(sql);
}

// Each entrypoint is below the limit; their shared execution site must cap the union.
export function entry01() { return execute('SELECT 1'); }
export function entry02() { return execute('SELECT 2'); }
export function entry03() { return execute('SELECT 3'); }
export function entry04() { return execute('SELECT 4'); }
export function entry05() { return execute('SELECT 5'); }
export function entry06() { return execute('SELECT 6'); }
export function entry07() { return execute('SELECT 7'); }
export function entry08() { return execute('SELECT 8'); }
export function entry09() { return execute('SELECT 9'); }
export function entry10() { return execute('SELECT 10'); }
export function entry11() { return execute('SELECT 11'); }
export function entry12() { return execute('SELECT 12'); }
export function entry13() { return execute('SELECT 13'); }
export function entry14() { return execute('SELECT 14'); }
export function entry15() { return execute('SELECT 15'); }
export function entry16() { return execute('SELECT 16'); }
export function entry17() { return execute('SELECT 17'); }
