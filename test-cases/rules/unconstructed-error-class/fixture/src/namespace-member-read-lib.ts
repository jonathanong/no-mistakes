// `namespace-member-read.ts` reads `version` (twice, once through a string) and
// `ReadHanded` and `ReadLiteral` through a namespace import, and never
// `ReadKept`.
export const version = 1;

export namespace ReadKept {
  export class ReadKeptDead extends Error {}
}

export namespace ReadHanded {
  export class ReadHandedDead extends Error {}
}

export namespace ReadLiteral {
  export class ReadLiteralDead extends Error {}
}
