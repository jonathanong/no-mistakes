import { query } from "@example/db";
import { getFragment } from "./frag";

const sql = "SELECT id FROM topics".append(getFragment());

export function load() {
  return query(sql);
}
