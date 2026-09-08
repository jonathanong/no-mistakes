import { query } from "@data-stores/psql";

export function insert(leftIds: string[], rightIds: string[]) {
  return query(`
    INSERT INTO edges (left_id, right_id)
    SELECT input.left_id, input.right_id
    FROM unnest($1::uuid[], $2::uuid[]) AS input(left_id, right_id)
    ORDER BY input.left_id, input.right_id
    ON CONFLICT (left_id, right_id) DO NOTHING
  `, [leftIds, rightIds]);
}
