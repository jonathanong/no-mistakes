import sql from "sql-template-strings";
import { write } from "@example/db";

export async function updateTopic(id: string, title: string) {
  const query = sql`UPDATE topics SET title = ${title} WHERE id = ${id}`;
  query.append(sql` AND published = true`);
  return write(query);
}

declare const importedAssignments: string;

export async function updateProjectedTopic(id: string) {
  const query = sql`UPDATE topics SET `;
  query.append(importedAssignments);
  query.append(sql` WHERE id = ${id}`);
  return write(query);
}

export function updateProjectedTopicFluent() {
  return write(sql`UPDATE topics SET `.append(importedAssignments));
}
