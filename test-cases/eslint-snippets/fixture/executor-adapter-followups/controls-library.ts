import { type SessionExecutor, write } from "@example/db";
import { write as otherWrite } from "@example/other";
import sql from "@example/sql";
import * as helpers from "./helpers";

type RunOptions = { query?: SessionExecutor };

async function selectWidget(id: string, execute: SessionExecutor) {
  return execute(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

export async function sameParameter(query: SessionExecutor, id: string) {
  await selectWidget(id, (statement) => write(statement, statement));
}

export async function extraArgument(query: SessionExecutor, id: string) {
  await helpers.extraArgument(id, { query });
}

export async function shadowedWrite(query: SessionExecutor, id: string) {
  const write = (statement: unknown, options: unknown) => otherWrite(statement, options);
  await selectWidget(id, (statement) => write(statement, query));
}

export async function localExecutorName(query: SessionExecutor, id: string) {
  function write(statement: string, options: SessionExecutor) {
    return options(statement);
  }
  await selectWidget(id, (statement) => write(statement, query));
}

export async function otherModule(query: SessionExecutor, id: string) {
  await selectWidget(id, (statement) => otherWrite(statement, query));
}
