// A class and a namespace of one name in one body are a single value: reading
// `Inner` hands on the class and its namespace, so `Deep` may be built.
export namespace MergedBody {
  export class Inner extends Error {}

  export namespace Inner {
    export class Deep extends Error {}
  }

  export const handed = Inner;
}
