declare module "fixture-package" {
  type Ctx = import("./types.ts").Ctx;
  export function run(): string;
  export abstract class Entry<Env> {
    constructor(env: Env);
  }
}
