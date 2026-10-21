import { query } from "@example/db";
import { registry } from "./barrel";

export function list() {
  return query(`SELECT id FROM ${registry.users}`);
}
