import { query } from "@data-stores/psql";
import String from "./custom-tag";

function build() {
  return String.raw`SELECT 1`;
}

export function load() {
  return query(build());
}
