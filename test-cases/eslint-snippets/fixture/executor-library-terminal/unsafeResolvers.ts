import { write, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export interface QueryOptions {
  query?: SessionExecutor;
  readOnly?: boolean;
}

export async function dynamicStatement(statement: string, options: QueryOptions = {}) {
  await write(statement, options);
}

export async function undeclaredStatement(options: QueryOptions = {}) {
  await write(undefined as never, options);
}

async function selectWidget(id: string, execute: SessionExecutor) {
  return execute(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

export async function manyStatements(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => {
    void id;
    return write(statement, options);
  });
}

export async function transformsStatement(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => write(String(statement), options));
}

export async function asyncClosure(id: string, options: QueryOptions = {}) {
  return selectWidget(id, async (statement) => write(statement, options));
}

export async function generatorClosure(id: string, options: QueryOptions = {}) {
  return selectWidget(id, function* (statement) {
    yield write(statement, options);
  });
}

export async function twoParameters(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement, extra?: string) => write(statement, options));
}

export async function notAnArgument(id: string, options: QueryOptions = {}) {
  const adapter = (statement: string) => write(statement, options);
  return selectWidget(id, adapter);
}
