import type { Dep } from "./dep"; // type-only import

const placeholder: Dep | undefined = undefined;

export function use() {
  // Dynamic and require imports of the same resolvable module.
  const dynamic = import("./dep");
  const required = require("./dep");
  return [placeholder, dynamic, required];
}
