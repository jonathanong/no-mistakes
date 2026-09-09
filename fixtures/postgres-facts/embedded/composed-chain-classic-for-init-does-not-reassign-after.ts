import { query } from "@data-stores/psql";

function build() {
  return "SELECT 1";
}

export function load() {
  for (const build = () => "DELETE FROM users"; false; ) {
    void build;
  }
  return query(build());
}
