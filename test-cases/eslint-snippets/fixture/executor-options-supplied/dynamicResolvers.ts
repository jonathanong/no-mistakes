import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export interface QueryOptions {
  query?: SessionExecutor;
  readOnly?: boolean;
}

declare const dynamicText: string;
declare const external: any;

async function runDynamic(id: string, options: QueryOptions = {}) {
  await options.query?.(dynamicText + id);
}

async function runHole(id: object, options: QueryOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}

async function runScalar(id: string, options: QueryOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}

async function middle(id: string, options: QueryOptions = {}) {
  await runDynamic(id, options);
}

export async function oneLevel(id: string, options: QueryOptions = {}) {
  await runDynamic(id, options);
}

export async function twoLevels(id: string, options: QueryOptions = {}) {
  await middle(id, options);
}

export async function holeOneLevel(id: object, options: QueryOptions = {}) {
  await runHole(id, options);
}

export async function writtenArgument(id: string, options: QueryOptions = {}) {
  id = "changed";
  await runScalar(id, options);
}

export async function defaultedArgument(id: string = "x", options: QueryOptions = {}) {
  await runScalar(id, options);
}

export async function literalArgument(id: string, options: QueryOptions = {}) {
  await runScalar("fixed", options);
}

export async function unresolvedCallee(id: string, options: QueryOptions = {}) {
  await external(id, options);
}

async function runOptional(options: QueryOptions, id?: string) {
  await options.query?.(sql`SELECT ${id}`);
}

export async function missingArgument(options: QueryOptions, id?: string) {
  await runOptional(options, id);
}
