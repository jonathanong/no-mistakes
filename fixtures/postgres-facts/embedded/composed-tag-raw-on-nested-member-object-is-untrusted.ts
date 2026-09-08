import { query } from "@data-stores/psql";

const helpers = {
  templates: {
    raw(strings: TemplateStringsArray) {
      return strings.join("");
    },
  },
};

const text = helpers.templates.raw`SELECT id FROM topics`;

export function load() {
  return query(text);
}
