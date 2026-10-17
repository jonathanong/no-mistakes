import { query } from "@example/db";

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
