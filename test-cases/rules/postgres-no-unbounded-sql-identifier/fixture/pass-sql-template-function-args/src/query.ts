import { read, write, query as run } from "@example/db";
import sql from "sql-template-strings";

declare const options: { limit: number };

export async function fromUnnest(postId: string, topicIds: string[], query: unknown) {
  await write(sql`INSERT INTO cats (post_id, topic_id) SELECT ${postId}, topic_id FROM UNNEST(${topicIds}::uuid[]) AS input(topic_id)`, { query });
}

export async function ordinality(runId: string, xs: { kind: string }[], ys: { id: string }[]) {
  await run(sql`INSERT INTO cands (run_id, kind, entity_id, ordinal) SELECT ${runId}, c.kind, c.entity_id, c.ordinal FROM unnest(${xs.map((x) => x.kind)}::text[], ${ys.map((y) => y.id)}::uuid[]) WITH ORDINALITY AS c (kind, entity_id, ordinal)`);
}

export async function anyCte(ids: string[], token: string) {
  await read(sql`WITH picked AS (SELECT id FROM posts WHERE id = ANY(${ids}::uuid[])) SELECT id FROM picked WHERE token = ${token}`, options);
}

export async function fnJoin(rows: string[], chunk: string) {
  await run(sql`SELECT p.id FROM posts p JOIN generate_series(1, ${rows.length}) g ON true WHERE p.chunk = ${chunk}`);
}
