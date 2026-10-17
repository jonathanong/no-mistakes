import { query } from "@example/db";

declare const parts: string[];

function selectTopics() {
  return "SELECT id FROM topics";
}

const statement = selectTopics().append(...parts);
query(statement);
