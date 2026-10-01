export namespace Required {
  // Not flagged: `namespace-escape-use.ts` requires the module, so any member
  // can be built from the required object.
  export class RequiredDead extends Error {}
}
