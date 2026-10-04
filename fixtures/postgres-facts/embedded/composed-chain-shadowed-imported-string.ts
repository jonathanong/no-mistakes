import { query } from "@example/db";
import String from "./custom-tag";

function build() {
  return String.raw`SELECT 1`;
}

export function load() {
  return query(build());
}
