import { write, type QueryExecutor, type QueryOptions } from "@data-stores/psql";
import sql, { type Statement } from "sql-template-strings";
import { runWith, type LookalikeOptions } from "./other";

type Run = (statement: Statement) => Promise<unknown>;

async function viaLookalike(id: string, options: LookalikeOptions = {}) {
  await write(sql`SELECT id FROM widgets WHERE id = ${id}`, options);
}

async function queryDynamic(column: string, run: Run) {
  return run(`SELECT ${column} FROM widgets`);
}

async function queryRest(run: Run, ...rest: string[]) {
  return run(sql`SELECT ${rest.length}`);
}

async function queryUntyped(id: string, run) {
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function queryUnknown(id: string, run: unknown) {
  return (run as Run)(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function queryPattern(id: string, { run }: { run: Run }) {
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function queryWritten(id: string, run: Run) {
  run = run.bind(null);
  return run(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function closureDynamic(column: string, options: QueryOptions = {}) {
  return queryDynamic(column, (statement) => write(statement, options));
}

async function closureRest(options: QueryOptions = {}) {
  return queryRest((statement) => write(statement, options), "a");
}

async function closureUntyped(id: string, options: QueryOptions = {}) {
  return queryUntyped(id, (statement) => write(statement, options));
}

async function closureUnknown(id: string, options: QueryOptions = {}) {
  return queryUnknown(id, (statement) => write(statement, options));
}

async function closurePattern(id: string, options: QueryOptions = {}) {
  return queryPattern(id, { run: (statement) => write(statement, options) });
}

async function closureWritten(id: string, options: QueryOptions = {}) {
  return queryWritten(id, (statement) => write(statement, options));
}

async function closureSpread(ids: string[], options: QueryOptions = {}) {
  return queryDynamic(...ids, (statement) => write(statement, options));
}

async function closureImported(options: QueryOptions = {}) {
  return runWith((statement) => write(statement, options));
}

export async function reports(query: QueryExecutor, id: string) {
  await viaLookalike(id, { query });
  await closureDynamic(id, { query });
  await closureRest({ query });
  await closureUntyped(id, { query });
  await closureUnknown(id, { query });
  await closurePattern(id, { query });
  await closureWritten(id, { query });
  await closureSpread([id], { query });
  await closureImported({ query });
}
