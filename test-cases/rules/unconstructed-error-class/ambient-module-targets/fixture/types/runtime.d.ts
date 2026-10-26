// A declaration-only module supplies class identity without a paths override.
declare module 'fixture:runtime' {
  export abstract class WorkerEntrypoint<Env = unknown, Props = unknown> {}
  export class ExternalError extends Error {}
}
declare module 'fixture:opaque' {
  export const Base: new () => unknown;
}
declare module 'fixture:duplicate' { export class Base {} }
declare module 'fixture:duplicate' { export class Base extends Error {} }
