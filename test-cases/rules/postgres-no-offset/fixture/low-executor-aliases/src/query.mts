import { beginTransaction } from '@example/db';
import type { QueryExecutor } from '@example/types';
import sql from 'sql-template-strings';
declare const dynamicSql: string;
type Local = QueryExecutor;
type Chained = Local;
type Transaction = Awaited<ReturnType<typeof beginTransaction>>;
type TransactionReturn = ReturnType<typeof beginTransaction>;

export function local(run: Chained) {
  run('SELECT 1'); // CLEAR
  run(sql`SELECT 1`); // CLEAR
  run(dynamicSql); // ADVISE
  run('SELECT * FROM users WHERE lower(id::text) = $1'); // TRANSFORMED
}
export function derived(run: Transaction) {
  run('SELECT 1'); // CLEAR
  run(dynamicSql); // ADVISE
}
export function returned(run: TransactionReturn) {
  run('SELECT 1'); // CLEAR
}
export async function transactionType() {
  function receiving(run: typeof transaction) {
    run('SELECT 1'); // CLEAR
    run(dynamicSql); // ADVISE
  }
  const transaction = await beginTransaction();
  receiving(transaction); // CLEAR
}
export function typedType(transaction: Local) {
  function receiving(run: typeof transaction) {
    run('SELECT 1'); // CLEAR
    run(dynamicSql); // ADVISE
  }
  receiving(transaction); // CLEAR
}
export async function mutable() {
  let transaction = await beginTransaction();
  function receiving(run: typeof transaction) {
    run('SELECT 1'); // ADVISE
  }
}
export function shadowed<Awaited>(run: Awaited<ReturnType<typeof beginTransaction>>) {
  run('SELECT 1'); // ADVISE
}
export function shadowedReturn<ReturnType>(run: ReturnType<typeof beginTransaction>) {
  run('SELECT 1'); // ADVISE
}
type Mixed = QueryExecutor | undefined;
export function mixed(run: Mixed) {
  run('SELECT 1'); // ADVISE
}
type Cycle = Cycle;
export function cycle(run: Cycle) { run('SELECT 1'); }
type Other = (value: string) => unknown;
export function unrelated(run: Other) { run('SELECT 1'); }

export function genericDirect(run: QueryExecutor<string>) {
  run('SELECT 1'); // CLEAR
}
import { beginTransaction as unconfiguredFactory } from '@unconfigured/db';
type Unconfigured = Awaited<ReturnType<typeof unconfiguredFactory>>;
export function unconfigured(run: Unconfigured) { run('SELECT 1'); }
