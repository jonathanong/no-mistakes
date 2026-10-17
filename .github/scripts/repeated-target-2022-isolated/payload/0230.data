import { query } from "@example/db";

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
