// `namespace-member-read.ts` reads `version` and `ReadHanded` through a
// namespace import, and never `ReadKept`.
export const version = 1;

export namespace ReadKept {
  export class ReadKeptDead extends Error {}
}

export namespace ReadHanded {
  export class ReadHandedDead extends Error {}
}
