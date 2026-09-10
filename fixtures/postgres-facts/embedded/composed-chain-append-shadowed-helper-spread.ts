import { query } from "@data-stores/psql";

declare const parts: string[];

function selectTopics() {
  return "SELECT id FROM topics";
}

export function load(selectTopics: () => string) {
  const statement = selectTopics().append(...parts);
  return query(statement);
}
