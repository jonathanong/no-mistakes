import { rows } from "./rows";

export function add(row) {
  rows.push(row);
  return rows;
}
