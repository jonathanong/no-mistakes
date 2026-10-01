// `class-shadow.ts` hides each of these names with a declaration in a namespace
// body, so it builds none of the `*Dead` classes and extends none of the bases,
// except where the name is private to one block of a merged namespace and
// built in another, which still names the import.
export namespace ClassErrors {
  export class ClassDead extends Error {}

  export class ClassBase extends Error {}
}

export namespace FunctionErrors {
  export class FunctionDead extends Error {}
}

export namespace EnumErrors {
  export class EnumDead extends Error {}
}

export namespace LaterErrors {
  export class LaterDead extends Error {}
}

export namespace NestedErrors {
  export class NestedDead extends Error {}
}

export namespace AliasErrors {
  export class AliasDead extends Error {}
}

export namespace MergedErrors {
  export class MergedDead extends Error {}
}

export class BareImported extends Error {}

export namespace SharedErrors {
  export class SharedDead extends Error {}
}

export namespace DeepErrors {
  export class DeepDead extends Error {}
}

export namespace DottedErrors {
  export class DottedDead extends Error {}
}

export namespace SplitErrors {
  export class SplitDead extends Error {}
}
