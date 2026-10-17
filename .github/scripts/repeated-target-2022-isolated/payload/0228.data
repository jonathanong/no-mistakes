import { query } from "@example/db";

const String: { raw?: () => string } = () => "";
String.raw = () => "DELETE FROM users";

function build() {
  return String.raw`SELECT 1`;
}

export function load() {
  return query(build());
}
