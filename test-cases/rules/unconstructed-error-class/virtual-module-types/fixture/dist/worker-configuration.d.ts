// Discovery skips `dist/`, so this script is read only because `types` names it.
declare namespace GeneratedModule {
  export abstract class WorkerEntrypoint<Env = unknown> {}
  export class GeneratedError extends Error {}
}
declare module 'fixture:generated' {
  export = GeneratedModule;
}
