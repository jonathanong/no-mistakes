import { query } from "@data-stores/psql";

declare const parts: string[];

function selectTopics() {
  return "SELECT id FROM topics";
}

const statement = selectTopics().append(...parts);
query(statement);
