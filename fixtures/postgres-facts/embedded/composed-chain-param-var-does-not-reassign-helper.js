import { query } from "@data-stores/psql";

function build() {
  return "SELECT 1";
}

function touch(build) {
  // JS: `var` + param share a binding. TypeScript reports a duplicate name.
  var build = () => "DELETE FROM users";
  void build;
}

export function load() {
  touch(() => "ignored");
  return query(build());
}
