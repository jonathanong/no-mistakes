import type { x } from "./dep"; // type import

export function use() {
  // Dynamic and require imports of the same resolvable module.
  const dynamic = import("./dep");
  const required = require("./dep");
  return [x, dynamic, required];
}
