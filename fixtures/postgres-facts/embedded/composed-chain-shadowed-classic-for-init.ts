import { query } from "@data-stores/psql";

function build() {
  return "SELECT 1";
}

export function load(active: boolean) {
  for (const build = () => "DELETE FROM users"; active; ) {
    return query(build());
  }
  return query(build());
}
