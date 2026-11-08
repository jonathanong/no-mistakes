declare module "typed-runtime-file" {
  type Ctx = typeof import("./types.ts");
  export function run(): string;
}
