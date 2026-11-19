declare module "fake-assert" {
  function assert(condition: unknown, statusCode: number, message?: string): asserts condition;
  export = assert;
}
