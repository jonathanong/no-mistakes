import { query } from "@example/db";

const helpers = {
  foo(strings: TemplateStringsArray) {
    return strings.join("");
  },
};

const text = helpers.foo`SELECT id FROM topics`;

export function load() {
  return query(text);
}
