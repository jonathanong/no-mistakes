import { query } from "@example/db";

function safe() {
  return "TRUSTED SQL";
}

function wrap(
  { ...rest }: Record<string, unknown>,
  [safe = () => "fallback", ...tail]: Array<() => string>,
) {
  return safe();
}

const sql = wrap({}, [() => "untrusted"]);

export function load() {
  return query(sql);
}
