import { query } from "@example/db";

declare const parts: string[];

function selectTopics() {
  return "SELECT id FROM topics";
}

export function load(selectTopics: () => string) {
  const statement = selectTopics().append(...parts);
  return query(statement);
}
