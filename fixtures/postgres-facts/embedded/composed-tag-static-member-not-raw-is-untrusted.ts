import { query } from "@data-stores/psql";

const helpers = {
  foo(strings: TemplateStringsArray) {
    return strings.join("");
  },
};

const text = helpers.foo`SELECT id FROM topics`;

export function load() {
  return query(text);
}
