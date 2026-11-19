import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as resolvers from "./dynamicResolvers";

interface QueryOptions {
  label?: string;
  query?: SessionExecutor;
  readOnly?: boolean;
}

declare const dynamicText: string;

async function runDynamic(id: string, options: QueryOptions = {}) {
  await options.query?.(dynamicText + id);
}

async function runHole(id: object, options: QueryOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}

async function oneLevel(id: string, options: QueryOptions = {}) {
  await runDynamic(id, options);
}

async function twoLevels(id: string, options: QueryOptions = {}) {
  await oneLevel(id, options);
}

async function holeOneLevel(id: object, options: QueryOptions = {}) {
  await runHole(id, options);
}

export async function forwarding(query: SessionExecutor, id: string, holder: object) {
  await oneLevel(id, { query });
  await twoLevels(id, { query });
  await holeOneLevel(holder, { query });
  await resolvers.oneLevel(id, { query });
  await resolvers.twoLevels(id, { query });
  await resolvers.holeOneLevel(holder, { query });
  await resolvers.writtenArgument(id, { query });
  await resolvers.defaultedArgument(id, { query });
  await resolvers.literalArgument(id, { query });
  await resolvers.unresolvedCallee(id, { query });
  await resolvers.missingArgument({ query });
}
