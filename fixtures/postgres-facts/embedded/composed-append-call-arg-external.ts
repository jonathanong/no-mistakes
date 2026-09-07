import { query } from "@data-stores/psql";
import { getFragment } from "./frag";

const sql = "SELECT id FROM topics".append(getFragment());

export function load() {
  return query(sql);
}
