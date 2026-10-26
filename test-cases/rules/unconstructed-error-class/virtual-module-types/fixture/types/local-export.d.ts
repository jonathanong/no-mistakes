// In-project `export =` namespace. No path mapping. Also named by `types`, so
// the type index must reuse this file's prepared facts.
declare namespace LocalModule {
  export class LocalBase {}
  export class LocalError extends Error {}
}
declare module 'fixture:local' {
  export = LocalModule;
}
