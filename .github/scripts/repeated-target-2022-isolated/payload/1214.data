export namespace Local {
  class Executor<T> {}
  export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('local'); }
}
// Type imports are file bindings even when written after the annotation.
export async function imported(query: Executor<string>) { for (const value of ['sql']) await query('imported'); } // selected
import type { Executor } from './executor.mts';
