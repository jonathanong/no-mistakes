import { query } from "@data-stores/psql";

function build() {
  return "SELECT * FROM topics";
}

export function run(load: () => string) {
  try {
    load();
  } catch (build) {
    return query(build());
  }
}
