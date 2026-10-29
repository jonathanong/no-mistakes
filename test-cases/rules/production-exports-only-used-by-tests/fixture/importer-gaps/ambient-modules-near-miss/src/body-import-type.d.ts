// A wildcard body with an import type can reference a production export.
declare module "*.css" {
  export const v: typeof import("./api.mjs").value;
}
