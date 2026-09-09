import { query } from "@data-stores/psql";

function build() {
  return "SELECT 1";
}

function touch(build: () => string) {
  build = () => "DELETE FROM users";
  void build;
}

export function load() {
  touch(() => "ignored");
  return query(build());
}
