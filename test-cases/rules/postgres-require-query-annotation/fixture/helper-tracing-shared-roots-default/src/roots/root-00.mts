import { query } from "@bench/db";
import { runAnnotated } from "../shared/helpers.mjs";
import { annotatedTag } from "../shared/constants.mjs";

export function handle00() {
  runAnnotated("items_00", (s) => query(s));
  return query(annotatedTag + "SELECT 0");
}
