import { query } from "@data-stores/psql";

export function load() {
  for (var q = "SELECT 1"; false; ) {
    void q;
  }
  return query(q);
}
