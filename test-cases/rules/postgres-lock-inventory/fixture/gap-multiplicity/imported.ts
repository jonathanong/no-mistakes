import { query } from "@example/db";
import { first, second } from "./statements";

export function importedGaps() {
  query(`${first}`); return query(`${second}`);
}
