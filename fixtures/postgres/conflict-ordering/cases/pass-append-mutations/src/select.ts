import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

export async function listTopics(scope: string) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(sql` AND tenant_id = ${scope}`);
  query.append(sql` ORDER BY id LIMIT 50`);
  return read(query);
}

declare const importedProjection: string;

export async function listProjectedTopics() {
  const query = sql`/* listProjectedTopics */ SELECT `;
  query.append(importedProjection);
  query.append(sql` FROM topics ORDER BY id LIMIT 50`);
  return read(query);
}

export function listProjectedTopicsFluent() {
  return read(sql`SELECT `.append(importedProjection));
}
