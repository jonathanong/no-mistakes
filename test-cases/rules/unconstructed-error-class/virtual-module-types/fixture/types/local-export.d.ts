// In-project `export =` namespace. No path mapping and not a `types` entry.
declare namespace LocalModule {
  export class LocalBase {}
  export class LocalError extends Error {}
}
declare module 'fixture:local' {
  export = LocalModule;
}
