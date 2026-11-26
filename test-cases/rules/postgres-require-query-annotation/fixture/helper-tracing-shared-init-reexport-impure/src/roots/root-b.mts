import { query } from "@bench/db";
import { namedTag } from "../shared/leaf-named.mjs";
import { header } from "../shared/consumer.mjs";

export function runB() {
  const seed = namedTag;
  const _header = header;
  return query(seed + " SELECT 1");
}
