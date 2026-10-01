// A class and a namespace of one name in one body are a single value: reading
// `Inner` hands on the class and the classes of its namespace.
export namespace MergedBody {
  export class Inner extends Error {}

  export namespace Inner {
    export class MergedDeep extends Error {}
  }

  export const handed = Inner;
}
