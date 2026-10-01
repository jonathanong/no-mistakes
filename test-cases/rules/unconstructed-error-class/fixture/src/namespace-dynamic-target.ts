export namespace Dynamic {
  // Not flagged: `namespace-escape-use.ts` imports the module at runtime, so
  // any member can be built from the loaded module.
  export class DynamicDead extends Error {}
}
