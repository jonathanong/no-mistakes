import type { Executor } from './executor.mts';
export namespace Local {
  class Executor<T> {}
  export async function execute(ids: string[], query: Executor<string>) {
    for (const id of ids) await (query as unknown as (sql: string) => Promise<unknown>)(id);
  }
}
export async function imported(ids: string[], query: Executor<string>) {
  for (const id of ids) await query(id);
}
// The whole type scope, including later declarations, hides the import.
export namespace LaterClass {
  export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('later'); }
  class Executor<T> {}
}
export namespace ExportedClass {
  export class Executor<T> {}
  export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('exported'); }
}
export namespace Alias {
  export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('alias'); }
  type Executor<T> = { value?: T };
}
export namespace Interface {
  export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('interface'); }
  interface Executor<T> { value?: T }
}
export namespace ExportedAlias {
  export type Executor<T> = { value?: T };
  export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('exported-alias'); }
}
export namespace ExportedInterface {
  export interface Executor<T> { value?: T }
  export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('exported-interface'); }
}
export namespace Enumeration {
  enum Executor { Value }
  export async function run(query: Executor) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('enum'); }
}
export namespace ExportedEnumeration {
  export enum Executor { Value }
  export async function run(query: Executor) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('exported-enum'); }
}
export namespace Outer {
  class Executor<T> {}
  export namespace Inner {
    export async function run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('nested'); }
  }
}
export async function parameter<Executor>(query: Executor) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('parameter'); }
export const Holder = class Executor<T> {
  async run(query: Executor<string>) { for (const value of ['sql']) await (query as unknown as (sql: string) => Promise<unknown>)('self'); }
};
// A value-only declaration does not occupy the imported type's namespace.
export namespace ValueOnly {
  const Executor = 1;
  export async function run(query: Executor<string>) { for (const value of ['sql']) await query('value-only'); } // selected
}
// Leaving a namespace/class scope must restore the genuine imported annotation.
export namespace Sibling {
  export async function run(query: Executor<string>) { for (const value of ['sql']) await query('sibling'); } // selected
}
export async function globalUtility(query: NonNullable<Executor<string>>) { for (const value of ['sql']) await query('utility'); } // selected
export async function indexed(query: { run: Executor<string> }['run']) { for (const value of ['sql']) await query('indexed'); } // selected
import type { Executor as OtherExecutor } from './other.mts';
export async function other(query: OtherExecutor<string>) { for (const value of ['sql']) await query('other'); }
// Preserve the canonical incoming edge beside annotation-only provenance.
function helper() {}
export function canonical() { helper(); }
