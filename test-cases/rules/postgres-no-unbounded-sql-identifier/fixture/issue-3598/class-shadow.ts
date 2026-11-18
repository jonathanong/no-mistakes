import { read } from "@example/db";
const ALLOWED = ["painted", "plain"] as const;
export function classShadow(table: string) {
  class ALLOWED {
    static some(predicate: unknown) { return true; }
  }
  if (!ALLOWED.some(item => item === table)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${table} LIMIT 10`);
}
export function innerGuardDoesNotProveOuter(table: string, input: string) {
  {
    const table = input;
    if (!ALLOWED.some(item => item === table)) throw new Error("Unknown variety");
  }
  return read(`SELECT id FROM ${table} LIMIT 10`);
}
export function classIncludes(table: string) {
  class ALLOWED {
    static includes(value: unknown) { return true; }
  }
  if (!ALLOWED.includes(table)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${table} LIMIT 10`);
}
