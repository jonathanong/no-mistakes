import { query, write } from "@example/db";

export async function fromUnnest(postId: string, topicIds: string[]) {
  await write(`INSERT INTO cats (post_id, topic_id) SELECT $1, topic_id FROM UNNEST($2::uuid[]) AS input(topic_id)`, [postId, topicIds], { query });
}

export async function ordinality(runId: string, kinds: string[], ids: string[]) {
  await query(`INSERT INTO cands (run_id, kind, entity_id) SELECT $1, c.kind, c.entity_id FROM unnest($2::text[], $3::uuid[]) WITH ORDINALITY AS c (kind, entity_id, ordinal)`, [runId, kinds, ids]);
}

export async function anyCte(ids: string[]) {
  await query(`WITH picked AS (SELECT id FROM posts WHERE id = ANY($1::uuid[])) SELECT id FROM picked`, [ids]);
}

export async function valueHoles(ids: string[], limit: number) {
  await query(`SELECT id FROM posts WHERE id = ANY(${ids}::uuid[]) LIMIT ${limit}`);
}

export async function rawUnnest(ids: string[]) {
  await query(`SELECT u.id FROM unnest(${ids}::uuid[]) AS u(id)`);
}

export async function rawFn(count: number) {
  await query(`SELECT g FROM generate_series(1, ${count}) g`);
}
