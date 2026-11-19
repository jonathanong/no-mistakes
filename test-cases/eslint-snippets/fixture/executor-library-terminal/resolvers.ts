import { read, write, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export interface QueryOptions {
  query?: SessionExecutor;
  readOnly?: boolean;
}

export async function passOptions(id: string, options: QueryOptions = {}) {
  await write(sql`UPDATE widgets SET state = 'sent' WHERE id = ${id}`, options);
}

export async function spreadOverride(id: string, options: QueryOptions = {}) {
  await write(sql`UPDATE widgets SET state = 'sent' WHERE id = ${id}`, {
    ...options,
    readOnly: false,
  });
}

export async function optionalNoDefault(id: string, options?: QueryOptions) {
  await read(sql`SELECT id FROM widgets WHERE id = ${id}`, options);
}

async function selectWidget(id: string, execute: SessionExecutor) {
  return execute(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

export async function fromExpression(id: string, options?: QueryOptions) {
  return selectWidget(id, (statement) => write(statement, { ...options, readOnly: false }));
}

export async function fromBlock(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => {
    return write(statement, options);
  });
}

export async function fromFunction(id: string, options: QueryOptions = {}) {
  return selectWidget(id, function (statement) {
    return write(statement, options);
  });
}
