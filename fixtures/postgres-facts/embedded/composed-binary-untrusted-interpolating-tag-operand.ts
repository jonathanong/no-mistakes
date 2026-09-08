import { query } from "@data-stores/psql";

const untrusted = "ignored";

const text = "SELECT * FROM topics WHERE id = " + unsafeTag`${untrusted}`;

export function load() {
  return query(text);
}

function unsafeTag(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
