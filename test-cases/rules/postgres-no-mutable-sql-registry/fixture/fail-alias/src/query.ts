import { query } from "@example/db";
import { tables as registry } from "./tables";

export function list() {
  return query(`SELECT id FROM ${registry.users}`);
}
