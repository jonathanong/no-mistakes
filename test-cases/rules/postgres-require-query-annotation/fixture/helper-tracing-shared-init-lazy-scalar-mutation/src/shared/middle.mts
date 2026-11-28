import { sqlPrefix } from "./lazy.mjs";

export const version = "/* ok */ ";

export function makeSql(body: string) {
  return sqlPrefix + body;
}
