import { read } from './db.mjs';
import { pageMember, pageAwaited, pageObjectRest, pageArrayRest, pageArrayDefault } from './pager.mjs';
import { sql } from './sql.mjs';
export const member = (after?: string) => pageMember(sql`/* member */ SELECT id FROM widgets`, after, statement => read(statement).then(() => ({ rows: [] })));
export const awaited = (after?: string) => pageAwaited(sql`/* awaited */ SELECT id FROM widgets`, after, statement => read(statement).then(() => ({ rows: [] })));
export const objectRest = (after?: string) => pageObjectRest(sql`/* objectRest */ SELECT id FROM widgets`, after, statement => read(statement).then(() => ({ rows: [] })));
export const arrayRest = (after?: string) => pageArrayRest(sql`/* arrayRest */ SELECT id FROM widgets`, after, statement => read(statement).then(() => [1]));
export const arrayDefault = (after?: string) => pageArrayDefault(sql`/* arrayDefault */ SELECT id FROM widgets`, after, statement => read(statement).then(() => [1]));
