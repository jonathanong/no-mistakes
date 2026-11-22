import { query } from "@example/db";
import { first, second } from "./statement";

export function lock(flag: boolean) {
  return query(flag ? `${first}` : `${second}`);
}
