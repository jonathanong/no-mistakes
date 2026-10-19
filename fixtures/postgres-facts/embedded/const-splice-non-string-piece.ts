import { read } from "@example/db";

const query = `SELECT ${1}`;

export function list() {
  return read(query);
}
